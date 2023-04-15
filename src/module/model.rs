use super::bindings_gen as bg;
use super::error::{ComError, ModuleError};

use libc::c_void;
use std::ffi::{c_char, CStr, CString};
use std::ptr;

pub const VERSION: u8 = 1;

#[derive(Debug)]
pub enum ConnParamType {
    Bool,
    Int,
    Float,
    String,
}

impl From<bg::ConnParamType> for ConnParamType {
    fn from(v: bg::ConnParamType) -> Self {
        match v {
            bg::ConnParamType::ConnParamBool => ConnParamType::Bool,
            bg::ConnParamType::ConnParamInt => ConnParamType::Int,
            bg::ConnParamType::ConnParamFloat => ConnParamType::Float,
            bg::ConnParamType::ConnParamString => ConnParamType::String,
        }
    }
}

#[derive(Debug)]
pub struct ConnParamInfo {
    pub name: String,
    pub typ: ConnParamType,
}

pub type DeficeInfoRec = Result<Vec<ConnParamInfo>, ModuleError>;

fn device_connect_info(res: *mut DeficeInfoRec, info: *const bg::DeviceConnectInfo) {
    if info.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("device_connect_info"));
        }
        return;
    }

    if unsafe { (*info).connection_params }.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer(
                "device_connect_info.connection_params",
            ));
        }
        return;
    }

    let len = unsafe { (*info).connection_params_len as usize };
    let mut device_connect_info: Vec<ConnParamInfo> = Vec::with_capacity(len);

    let s = unsafe { std::slice::from_raw_parts((*info).connection_params, len) };

    for i in s {
        if i.name.is_null() {
            unsafe {
                *res = Err(ModuleError::InvalidPointer(
                    "device_connect_info.connection_params[i].name",
                ));
            }
            return;
        }

        match unsafe { CStr::from_ptr(i.name).to_str() } {
            Err(err) => {
                unsafe {
                    *res = Err(ModuleError::StrError(err.into()));
                }
                return;
            }
            Ok(s) => {
                device_connect_info.push(ConnParamInfo {
                    name: s.to_string(),
                    typ: i.typ.into(),
                });
            }
        }
    }

    unsafe {
        *res = Ok(device_connect_info);
    }
}

pub struct Handle(*const c_void);

impl Handle {
    pub fn new() -> Self {
        Handle(std::ptr::null())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn handler_ptr(&mut self) -> *mut *mut c_void {
        self as *mut Self as *mut *mut c_void
    }

    pub fn handler(&mut self) -> *mut c_void {
        self.0 as _
    }
}

unsafe impl Send for Handle {}

pub extern "C" fn device_info_callback(obj: *mut c_void, info: *mut bg::DeviceConnectInfo) {
    device_connect_info(obj as _, info);
}

pub enum ConnParamValType {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

impl ConnParamValType {
    pub fn parse_cstr(&self) -> CString {
        let str = match self {
            ConnParamValType::Bool(val) => val.to_string(),
            ConnParamValType::Int(val) => val.to_string(),
            ConnParamValType::Float(val) => val.to_string(),
            ConnParamValType::String(val) => val.clone(),
        };

        CString::new(str).unwrap()
    }
}

pub struct ConnParamValue {
    val: ConnParamValType,
    val_parsed: Option<CString>,
}

impl ConnParamValue {
    pub fn new(val: ConnParamValType) -> Self {
        Self {
            val,
            val_parsed: None,
        }
    }

    pub fn parsed(&mut self) -> &CString {
        if let Some(ref val) = self.val_parsed {
            return val;
        }

        self.val_parsed = Some(self.val.parse_cstr());

        self.val_parsed.as_ref().unwrap()
    }

    pub fn get(&self) -> &ConnParamValType {
        &self.val
    }
}

pub struct ConnParam {
    name: String,
    c_name: Option<CString>,
    value: ConnParamValue,
}

impl ConnParam {
    pub fn new(name: String, value: ConnParamValType) -> Self {
        Self {
            name,
            c_name: None,
            value: ConnParamValue::new(value),
        }
    }

    pub fn get_c_name(&mut self) -> &CString {
        if let Some(ref name) = self.c_name {
            return name;
        }

        self.c_name = Some(CString::new(self.name.clone()).unwrap());

        self.c_name.as_ref().unwrap()
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_value(&self) -> &ConnParamValType {
        self.value.get()
    }
}

pub struct DeviceConnectConf {
    params: Vec<ConnParam>,
    c_params: Option<Vec<bg::ConnParam>>,
}

impl DeviceConnectConf {
    pub fn new(params: Vec<ConnParam>) -> Self {
        Self {
            params,
            c_params: None,
        }
    }
}

impl From<&mut DeviceConnectConf> for bg::DeviceConnectConf {
    fn from(info: &mut DeviceConnectConf) -> Self {
        if let Some(ref c_params) = info.c_params {
            return bg::DeviceConnectConf {
                connection_params: c_params.as_ptr() as _,
                connection_params_len: c_params.len() as i32,
            };
        }

        let mut c_params = Vec::with_capacity(info.params.len());

        for param in info.params.iter_mut() {
            c_params.push(bg::ConnParam {
                name: param.get_c_name().as_ptr() as _,
                value: param.value.parsed().as_ptr() as _,
            });
        }

        info.c_params = Some(c_params);
        let c_params_ptr = info.c_params.as_ref().unwrap();

        bg::DeviceConnectConf {
            connection_params: c_params_ptr.as_ptr() as _,
            connection_params_len: c_params_ptr.len() as i32,
        }
    }
}

#[derive(Debug)]
pub struct DeviceConfInfoEntry {
    pub id: i32,
    pub name: String,
    pub data: DeviceConfInfoEntryType,
}

#[derive(Debug)]
pub enum DeviceConfInfoEntryType {
    Section(DeviceConfInfo),
    String(DeviceConfInfoEntryString),
    Int(DeviceConfInfoEntryInt),
    IntRange(DeviceConfInfoEntryIntRange),
    Float(DeviceConfInfoEntryFloat),
    FloatRange(DeviceConfInfoEntryFloatRange),
    JSON(DeviceConfInfoEntryJSON),
    ChoiceList(DeviceConfInfoEntryChoiceList),
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryString {
    pub required: bool,
    pub default: Option<String>,

    pub min_len: Option<i32>,
    pub max_len: Option<i32>,
    pub match_regex: Option<String>,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryInt {
    pub required: bool,
    pub default: Option<i32>,

    pub lt: Option<i32>,
    pub gt: Option<i32>,
    pub neq: Option<i32>,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryIntRange {
    pub required: bool,
    pub def_from: Option<i32>,
    pub def_to: Option<i32>,

    pub min: i32,
    pub max: i32,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryFloat {
    pub required: bool,
    pub default: Option<f32>,

    pub lt: Option<f32>,
    pub gt: Option<f32>,
    pub neq: Option<f32>,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryFloatRange {
    pub required: bool,
    pub def_from: Option<f32>,
    pub def_to: Option<f32>,

    pub min: f32,
    pub max: f32,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryJSON {
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryChoiceList {
    pub required: bool,
    pub default: Option<i32>,

    pub choices: Vec<String>,
}

#[derive(Debug)]
pub struct DeviceConfInfo {
    pub device_confs: Vec<DeviceConfInfoEntry>,
}

impl DeviceConfInfo {
    pub fn new(len: usize) -> Self {
        Self {
            device_confs: Vec::with_capacity(len),
        }
    }
}

fn build_device_conf_info(info: *mut bg::DeviceConfInfo) -> Result<DeviceConfInfo, ModuleError> {
    if unsafe { (*info).device_confs }.is_null() {
        return Err(ModuleError::InvalidPointer("device_conf_info.device_confs"));
    }

    let confs =
        unsafe { std::slice::from_raw_parts((*info).device_confs, (*info).device_confs_len as _) };
    let mut res = DeviceConfInfo::new(unsafe { (*info).device_confs_len } as _);

    for conf in confs {
        res.device_confs.push(DeviceConfInfoEntry {
            id: conf.id,
            name: str_from_c_char(conf.name),
            data: build_device_conf_info_entry_data(conf)?,
        });
    }

    Ok(res)
}

fn build_device_conf_info_entry_data(
    conf: &bg::DeviceConfInfoEntry,
) -> Result<DeviceConfInfoEntryType, ModuleError> {
    match conf.typ {
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeSection => {
            let section = build_device_conf_info(conf.data as *mut bg::DeviceConfInfo)?;

            Ok(DeviceConfInfoEntryType::Section(section))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeString => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryString) };

            Ok(DeviceConfInfoEntryType::String(DeviceConfInfoEntryString {
                required: data.required,
                default: option_str_from_c_char(data.def),
                min_len: nullable_into_option(data.min_len),
                max_len: nullable_into_option(data.max_len),
                match_regex: option_str_from_c_char(data.match_regex),
            }))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeInt => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryInt) };

            Ok(DeviceConfInfoEntryType::Int(DeviceConfInfoEntryInt {
                required: data.required,
                default: nullable_into_option(data.def),
                lt: nullable_into_option(data.lt),
                gt: nullable_into_option(data.gt),
                neq: nullable_into_option(data.neq),
            }))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeIntRange => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryIntRange) };

            Ok(DeviceConfInfoEntryType::IntRange(
                DeviceConfInfoEntryIntRange {
                    required: data.required,
                    def_from: nullable_into_option(data.def_from),
                    def_to: nullable_into_option(data.def_to),
                    min: data.min,
                    max: data.max,
                },
            ))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeFloat => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryFloat) };

            Ok(DeviceConfInfoEntryType::Float(DeviceConfInfoEntryFloat {
                required: data.required,
                default: nullable_into_option(data.def),
                lt: nullable_into_option(data.lt),
                gt: nullable_into_option(data.gt),
                neq: nullable_into_option(data.neq),
            }))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeFloatRange => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryFloatRange) };

            Ok(DeviceConfInfoEntryType::FloatRange(
                DeviceConfInfoEntryFloatRange {
                    required: data.required,
                    def_from: nullable_into_option(data.def_from),
                    def_to: nullable_into_option(data.def_to),
                    min: data.min,
                    max: data.max,
                },
            ))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeJSON => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryJSON) };

            Ok(DeviceConfInfoEntryType::JSON(DeviceConfInfoEntryJSON {
                required: data.required,
                default: option_str_from_c_char(data.def),
            }))
        }
        bg::DeviceConfInfoEntryType::DeviceConfInfoEntryTypeChoiceList => {
            let data = unsafe { *(conf.data as *mut bg::DeviceConfInfoEntryChoiceList) };

            let mut entry = DeviceConfInfoEntryChoiceList {
                required: data.required,
                default: nullable_into_option(data.def),
                choices: Vec::with_capacity(data.chioces_len as _),
            };

            for choice in unsafe { std::slice::from_raw_parts(data.choices, data.chioces_len as _) }
            {
                entry.choices.push(str_from_c_char(*choice));
            }

            Ok(DeviceConfInfoEntryType::ChoiceList(entry))
        }
    }
}

pub type DeviceConfInfoRec = Result<DeviceConfInfo, ModuleError>;

fn device_conf_info(res: *mut DeviceConfInfoRec, info: *mut bg::DeviceConfInfo) {
    if info.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("device_conf"));
        }
        return;
    }

    unsafe {
        *res = build_device_conf_info(info);
    }
}

pub extern "C" fn device_conf_info_callback(obj: *mut c_void, info: *mut bg::DeviceConfInfo) {
    device_conf_info(obj as _, info);
}

pub enum DeviceConfType {
    String(CString),
    Int(i32),
    IntRange([i32; 2]),
    Float(f32),
    FloatRange([f32; 2]),
    JSON(CString),
    ChoiceList(i32),
}

impl DeviceConfType {
    fn as_ptr(&self) -> *mut c_void {
        match self {
            DeviceConfType::String(s) => s.as_ptr() as _,
            DeviceConfType::Int(i) => i as *const i32 as _,
            DeviceConfType::IntRange(ir) => ir.as_ptr() as _,
            DeviceConfType::Float(f) => f as *const f32 as _,
            DeviceConfType::FloatRange(fr) => fr.as_ptr() as _,
            DeviceConfType::JSON(j) => j.as_ptr() as _,
            DeviceConfType::ChoiceList(cl) => cl as *const i32 as _,
        }
    }
}

fn ptr_from_device_conf_option(data: &Option<DeviceConfType>) -> *mut c_void {
    match data {
        Some(d) => d.as_ptr(),
        None => ptr::null::<i32>() as _,
    }
}

pub struct DeviceConfEntry {
    id: i32,
    data: Option<DeviceConfType>,
}

impl DeviceConfEntry {
    pub fn new(id: i32, data: Option<DeviceConfType>) -> Self {
        Self { id, data }
    }

    fn get_data(&self) -> &Option<DeviceConfType> {
        &self.data
    }
}

pub fn build_device_conf_entry_raw_vec(
    confs: &mut Vec<DeviceConfEntry>,
) -> Vec<bg::DeviceConfEntry> {
    let mut confs_raw = Vec::with_capacity(confs.len());
    for conf in confs.iter_mut() {
        confs_raw.push(bg::DeviceConfEntry {
            id: conf.id,
            data: ptr_from_device_conf_option(&conf.data),
        })
    }

    confs_raw
}

pub fn build_device_conf(confs: &Vec<bg::DeviceConfEntry>) -> bg::DeviceConf {
    bg::DeviceConf {
        confs: confs.as_ptr() as _,
        confs_len: confs.len() as _,
    }
}

#[derive(Debug)]
pub enum SensorDataType {
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Timestamp,
    String,
    JSON,
}

impl From<bg::SensorDataType> for SensorDataType {
    fn from(value: bg::SensorDataType) -> Self {
        match value {
            bg::SensorDataType::SensorDataTypeInt16 => SensorDataType::Int16,
            bg::SensorDataType::SensorDataTypeInt32 => SensorDataType::Int32,
            bg::SensorDataType::SensorDataTypeInt64 => SensorDataType::Int64,
            bg::SensorDataType::SensorDataTypeFloat32 => SensorDataType::Float32,
            bg::SensorDataType::SensorDataTypeFloat64 => SensorDataType::Float64,
            bg::SensorDataType::SensorDataTypeTimestamp => SensorDataType::Timestamp,
            bg::SensorDataType::SensorDataTypeString => SensorDataType::String,
            bg::SensorDataType::SensorDataTypeJSON => SensorDataType::JSON,
        }
    }
}

#[derive(Debug)]
pub struct SensorDataTypeInfo {
    pub name: String,
    pub typ: SensorDataType,
}

#[derive(Debug)]
pub struct SensorTypeInfo {
    pub name: String,
    pub data_type_infos: Vec<SensorDataTypeInfo>,
}

pub fn build_sensor_type_infos(
    infos: *mut bg::SensorTypeInfos,
) -> Result<Vec<SensorTypeInfo>, ModuleError> {
    let infos_slice = unsafe {
        std::slice::from_raw_parts(
            (*infos).sensor_type_infos,
            (*infos).sensor_type_infos_len as _,
        )
    };

    let mut res_infos = Vec::with_capacity(infos_slice.len());
    for info in infos_slice {
        let data_type_infos_slice = unsafe {
            std::slice::from_raw_parts(info.data_type_infos, info.data_type_infos_len as _)
        };

        let mut res_data_type_infos_slice = Vec::with_capacity(data_type_infos_slice.len());
        for data_type_info in data_type_infos_slice {
            res_data_type_infos_slice.push(SensorDataTypeInfo {
                name: str_from_c_char(data_type_info.name),
                typ: data_type_info.typ.into(),
            })
        }

        res_infos.push(SensorTypeInfo {
            name: str_from_c_char(info.name),
            data_type_infos: res_data_type_infos_slice,
        })
    }

    Ok(res_infos)
}

pub type SensorTypeInfosRec = Result<Vec<SensorTypeInfo>, ModuleError>;

fn sensor_type_infos(res: *mut SensorTypeInfosRec, infos: *mut bg::SensorTypeInfos) {
    if infos.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("device_conf"));
        }
        return;
    }

    unsafe {
        *res = build_sensor_type_infos(infos);
    }
}

pub extern "C" fn sensor_type_infos_callback(obj: *mut c_void, infos: *mut bg::SensorTypeInfos) {
    sensor_type_infos(obj as _, infos);
}

#[derive(Debug)]
pub struct Message {
    pub msg: MessageType,
}

impl From<bg::Message> for Message {
    fn from(value: bg::Message) -> Self {
        match value.typ {
            bg::MessageType::MessageTypeSensor => Self {
                msg: MessageType::Sensor(
                    unsafe { &(*(value.data as *const bg::SensorMsg)) }.into(),
                ),
            },
            bg::MessageType::MessageTypeCommon => Self {
                msg: MessageType::Common(
                    unsafe { &(*(value.data as *const bg::CommonMsg)) }.into(),
                ),
            },
        }
    }
}

#[derive(Debug)]
pub enum MessageType {
    Sensor(SensorMsg),
    Common(CommonMsg),
}

#[derive(Debug)]
pub struct SensorMsg {
    pub name: String,
    pub data: Vec<SensorMsgData>,
}

impl From<&bg::SensorMsg> for SensorMsg {
    fn from(value: &bg::SensorMsg) -> Self {
        let data_list = unsafe { std::slice::from_raw_parts(value.data, value.data_len as _) };

        Self {
            name: str_from_c_char(value.name),
            data: data_list.into_iter().map(|v| v.into()).collect(),
        }
    }
}

#[derive(Debug)]
pub struct SensorMsgData {
    pub name: String,
    pub data: SensorMsgDataType,
}

impl From<&bg::SensorMsgData> for SensorMsgData {
    fn from(value: &bg::SensorMsgData) -> Self {
        let data = unsafe {
            match value.typ {
                bg::SensorDataType::SensorDataTypeInt16 => {
                    SensorMsgDataType::Int16(*(value.data as *mut i16))
                }
                bg::SensorDataType::SensorDataTypeInt32 => {
                    SensorMsgDataType::Int32(*(value.data as *mut i32))
                }
                bg::SensorDataType::SensorDataTypeInt64 => {
                    SensorMsgDataType::Int64(*(value.data as *mut i64))
                }
                bg::SensorDataType::SensorDataTypeFloat32 => {
                    SensorMsgDataType::Float32(*(value.data as *mut f32))
                }
                bg::SensorDataType::SensorDataTypeFloat64 => {
                    SensorMsgDataType::Float64(*(value.data as *mut f64))
                }
                bg::SensorDataType::SensorDataTypeTimestamp => {
                    SensorMsgDataType::Timestamp(*(value.data as *mut i64))
                }
                bg::SensorDataType::SensorDataTypeString => {
                    SensorMsgDataType::String(str_from_c_char(value.data as *mut c_char))
                }
                bg::SensorDataType::SensorDataTypeJSON => {
                    SensorMsgDataType::JSON(str_from_c_char(value.data as *mut c_char))
                }
            }
        };

        Self {
            name: str_from_c_char(value.name),
            data,
        }
    }
}

#[derive(Debug)]
pub enum SensorMsgDataType {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Timestamp(i64),
    String(String),
    JSON(String),
}

#[derive(Debug)]
pub struct CommonMsg {
    code: MsgCode,
    msg: String,
}

impl From<&bg::CommonMsg> for CommonMsg {
    fn from(value: &bg::CommonMsg) -> Self {
        Self {
            code: value.code.into(),
            msg: str_from_c_char(value.msg),
        }
    }
}

#[derive(Debug)]
pub enum MsgCode {
    Info,
    Warn,
    Error,
}

impl From<bg::MsgCode> for MsgCode {
    fn from(value: bg::MsgCode) -> Self {
        match value {
            bg::MsgCode::MsgCodeInfo => MsgCode::Info,
            bg::MsgCode::MsgCodeWarn => MsgCode::Warn,
            bg::MsgCode::MsgCodeError => MsgCode::Error,
        }
    }
}

pub trait MsgHandler: Send + Sync {
    fn handle_msg(&self, msg: Message);
}

pub struct MsgHandle(Box<dyn MsgHandler>);

impl MsgHandle {
    pub fn new<H: MsgHandler + 'static>(msg_handler: H) -> Self {
        Self(Box::new(msg_handler))
    }
}

pub extern "C" fn handle_msg_callback(handler: *mut c_void, msg_data: bg::Message) {
    let h = handler as *const MsgHandle;

    unsafe {
        (*h).0.handle_msg(msg_data.into());
    }
}

// ------------------- Utility functions -------------------

pub fn convert_com_error(err: u8) -> Result<(), ComError> {
    match err {
        0 => Ok(()),
        1 => Err(ComError::ConnectionError),
        2 => Err(ComError::InvalidArgument),
        _ => Err(ComError::Unknown),
    }
}

fn nullable_into_option<T: Copy>(nullable_val: *mut T) -> Option<T> {
    if nullable_val.is_null() {
        None
    } else {
        Some(unsafe { *nullable_val })
    }
}

fn option_str_from_c_char(nullable_raw: *mut c_char) -> Option<String> {
    if nullable_raw.is_null() {
        None
    } else {
        Some(str_from_c_char(nullable_raw))
    }
}

fn str_from_c_char(raw: *mut c_char) -> String {
    let cstr = unsafe { CStr::from_ptr(raw) };

    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

use super::bindings_gen as bg;
use super::error::{ComError, ModuleError};

use libc::c_void;
use std::ffi::{c_char, CStr, CString};

pub const VERSION: u8 = 1;

#[derive(Debug)]
enum ConnParamType {
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
pub struct ConnParamConf {
    name: String,
    typ: ConnParamType,
}

pub type DeficeConfRec = Result<Vec<ConnParamConf>, ModuleError>;

fn device_connect_info(res: *mut DeficeConfRec, info: *const bg::DeviceConnectInfo) {
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
    let mut device_connect_info: Vec<ConnParamConf> = Vec::with_capacity(len);

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
                device_connect_info.push(ConnParamConf {
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
    pub fn parse_cstr(&self) -> Box<CString> {
        let str = match self {
            ConnParamValType::Bool(val) => val.to_string(),
            ConnParamValType::Int(val) => val.to_string(),
            ConnParamValType::Float(val) => val.to_string(),
            ConnParamValType::String(val) => val.clone(),
        };

        Box::new(CString::new(str).unwrap())
    }
}

pub struct ConnParamValue {
    val: ConnParamValType,
    val_parsed: Option<Box<CString>>,
}

impl ConnParamValue {
    pub fn new(val: ConnParamValType) -> Self {
        Self {
            val,
            val_parsed: None,
        }
    }

    pub fn parsed(&mut self) -> &Box<CString> {
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
    required: bool,
    default: Option<String>,

    min_len: Option<i32>,
    max_len: Option<i32>,
    match_regex: Option<String>,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryInt {
    required: bool,
    default: Option<i32>,

    lt: Option<i32>,
    gt: Option<i32>,
    neq: Option<i32>,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryIntRange {
    required: bool,
    def_from: Option<i32>,
    def_to: Option<i32>,

    min: i32,
    max: i32,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryFloat {
    required: bool,
    default: Option<f32>,

    lt: Option<f32>,
    gt: Option<f32>,
    neq: Option<f32>,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryFloatRange {
    required: bool,
    def_from: Option<f32>,
    def_to: Option<f32>,

    min: f32,
    max: f32,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryJSON {
    required: bool,
    default: Option<String>,
}

#[derive(Debug)]
pub struct DeviceConfInfoEntryChoiceList {
    required: bool,
    default: Option<i32>,

    choices: Vec<String>,
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

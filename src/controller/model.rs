use std::{collections::HashMap, ffi::CString};

use crate::{module, service};

use sqlx::types::Json;

pub mod internal {
    use crate::{controller::msg, module, service};

    // TODO: issue #81
    // pub enum DeviceState {
    //     Inited,
    //     Connected,
    //     Configurated,
    // }

    pub struct Device {
        pub id: service::DeviceID,
        pub module: module::Module,
        pub msg_handler: Option<msg::Handler>,
        // TODO: issue #81
        // pub state: DeviceState,
    }
}

#[derive(Debug)]
pub struct DeviceInitData {
    pub id: i32,
    pub conn_params: Vec<ConnParamConf>,
}

#[derive(Debug)]
pub enum ConnParamType {
    Bool,
    Int,
    Float,
    String,
    ChoiceList,
}

impl From<module::ConnParamType> for ConnParamType {
    fn from(value: module::ConnParamType) -> Self {
        match value {
            module::ConnParamType::Bool => ConnParamType::Bool,
            module::ConnParamType::Int => ConnParamType::Int,
            module::ConnParamType::Float => ConnParamType::Float,
            module::ConnParamType::String => ConnParamType::String,
            module::ConnParamType::ChoiceList => ConnParamType::ChoiceList,
        }
    }
}

#[derive(Debug)]
pub enum ConnParamEntryInfo {
    ChoiceList(ConnParamChoiceListInfo),
}

impl From<module::ConnParamEntryInfo> for ConnParamEntryInfo {
    fn from(value: module::ConnParamEntryInfo) -> Self {
        match value {
            module::ConnParamEntryInfo::ChoiceList(v) => {
                ConnParamEntryInfo::ChoiceList(ConnParamChoiceListInfo { choices: v.choices })
            }
        }
    }
}

#[derive(Debug)]
pub struct ConnParamChoiceListInfo {
    pub choices: Vec<String>,
}

#[derive(Debug)]
pub struct ConnParamConf {
    pub name: String,
    pub typ: ConnParamType,
    pub info: Option<ConnParamEntryInfo>,
}

impl From<module::ConnParamInfo> for ConnParamConf {
    fn from(value: module::ConnParamInfo) -> Self {
        Self {
            name: value.name,
            typ: value.typ.into(),
            info: value.info.map(|v| v.into()),
        }
    }
}

pub enum ConnParamValType {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

impl From<ConnParamValType> for module::ConnParamValType {
    fn from(value: ConnParamValType) -> Self {
        match value {
            ConnParamValType::Bool(v) => module::ConnParamValType::Bool(v),
            ConnParamValType::Int(v) => module::ConnParamValType::Int(v),
            ConnParamValType::Float(v) => module::ConnParamValType::Float(v),
            ConnParamValType::String(v) => module::ConnParamValType::String(v),
        }
    }
}

pub struct ConnParam {
    pub name: String,
    pub value: ConnParamValType,
}

impl From<ConnParam> for module::ConnParam {
    fn from(value: ConnParam) -> Self {
        Self::new(value.name, value.value.into())
    }
}

pub type DeviceConnectConf = Vec<ConnParam>;

impl From<DeviceConnectConf> for module::DeviceConnectConf {
    fn from(mut value: DeviceConnectConf) -> Self {
        Self::new(value.drain(..).map(|v| v.into()).collect())
    }
}

#[derive(Debug)]
pub struct DeviceConfInfoEntry {
    pub id: i32,
    pub name: String,
    pub data: DeviceConfInfoEntryType,
}

impl From<module::DeviceConfInfoEntry> for DeviceConfInfoEntry {
    fn from(value: module::DeviceConfInfoEntry) -> Self {
        Self {
            id: value.id,
            name: value.name,
            data: value.data.into(),
        }
    }
}

pub type DeviceConfInfo = Vec<DeviceConfInfoEntry>;

impl From<module::DeviceConfInfo> for DeviceConfInfo {
    fn from(mut value: module::DeviceConfInfo) -> Self {
        value.device_confs.drain(..).map(|v| v.into()).collect()
    }
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

impl From<module::DeviceConfInfoEntryType> for DeviceConfInfoEntryType {
    fn from(value: module::DeviceConfInfoEntryType) -> Self {
        match value {
            module::DeviceConfInfoEntryType::Section(v) => {
                DeviceConfInfoEntryType::Section(v.into())
            }
            module::DeviceConfInfoEntryType::String(v) => {
                DeviceConfInfoEntryType::String(DeviceConfInfoEntryString {
                    required: v.required,
                    default: v.default,
                    min_len: v.min_len,
                    max_len: v.max_len,
                    match_regex: v.match_regex,
                })
            }
            module::DeviceConfInfoEntryType::Int(v) => {
                DeviceConfInfoEntryType::Int(DeviceConfInfoEntryInt {
                    required: v.required,
                    default: v.default,
                    lt: v.lt,
                    gt: v.gt,
                    neq: v.neq,
                })
            }
            module::DeviceConfInfoEntryType::IntRange(v) => {
                DeviceConfInfoEntryType::IntRange(DeviceConfInfoEntryIntRange {
                    required: v.required,
                    def_from: v.def_from,
                    def_to: v.def_to,
                    min: v.min,
                    max: v.max,
                })
            }
            module::DeviceConfInfoEntryType::Float(v) => {
                DeviceConfInfoEntryType::Float(DeviceConfInfoEntryFloat {
                    required: v.required,
                    default: v.default,
                    lt: v.lt,
                    gt: v.gt,
                    neq: v.neq,
                })
            }
            module::DeviceConfInfoEntryType::FloatRange(v) => {
                DeviceConfInfoEntryType::FloatRange(DeviceConfInfoEntryFloatRange {
                    required: v.required,
                    def_from: v.def_from,
                    def_to: v.def_to,
                    min: v.min,
                    max: v.max,
                })
            }
            module::DeviceConfInfoEntryType::JSON(v) => {
                DeviceConfInfoEntryType::JSON(DeviceConfInfoEntryJSON {
                    required: v.required,
                    default: v.default,
                })
            }
            module::DeviceConfInfoEntryType::ChoiceList(v) => {
                DeviceConfInfoEntryType::ChoiceList(DeviceConfInfoEntryChoiceList {
                    required: v.required,
                    default: v.default,
                    choices: v.choices,
                })
            }
        }
    }
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

pub enum DeviceConfType {
    String(String),
    Int(i32),
    IntRange([i32; 2]),
    Float(f32),
    FloatRange([f32; 2]),
    JSON(String),
    ChoiceList(i32),
}

impl From<DeviceConfType> for module::DeviceConfType {
    fn from(value: DeviceConfType) -> Self {
        match value {
            DeviceConfType::String(v) => module::DeviceConfType::String(CString::new(v).unwrap()),
            DeviceConfType::Int(v) => module::DeviceConfType::Int(v),
            DeviceConfType::IntRange(v) => module::DeviceConfType::IntRange(v),
            DeviceConfType::Float(v) => module::DeviceConfType::Float(v),
            DeviceConfType::FloatRange(v) => module::DeviceConfType::FloatRange(v),
            DeviceConfType::JSON(v) => module::DeviceConfType::JSON(CString::new(v).unwrap()),
            DeviceConfType::ChoiceList(v) => module::DeviceConfType::ChoiceList(v),
        }
    }
}

pub struct DeviceConfEntry {
    pub id: i32,
    pub data: Option<DeviceConfType>,
}

impl From<DeviceConfEntry> for module::DeviceConfEntry {
    fn from(value: DeviceConfEntry) -> Self {
        module::DeviceConfEntry::new(value.id, value.data.map(|v| v.into()))
    }
}

pub struct GetSensorDataPayload {
    pub device_id: i32,
    pub sensor: String,
    pub fields: Vec<String>,
    pub sort: Sort,
    pub from: Option<SensorData>,
    pub limit: Option<i32>,
}

impl GetSensorDataPayload {
    pub fn to_sensor_data_filter(&self) -> service::SensorDataFilter {
        let mut res = service::SensorDataFilter::default();

        if let Some(ref from) = self.from {
            if self.sort.order == SortOrder::ASC {
                res.to = Some((self.sort.field.clone(), from.clone().into()));
            } else {
                res.from = Some((self.sort.field.clone(), from.clone().into()));
            }
        }

        res.limit = self.limit.clone();
        res.sort = Some(self.sort.clone().into());

        res
    }
}

pub type GetSensorDataResult = Vec<HashMap<String, SensorData>>;

pub fn sensor_data_result_from_service(
    mut value: Vec<service::SensorDataRow>,
) -> GetSensorDataResult {
    value
        .drain(..)
        .map(|mut v| v.0.drain(..).map(|v| (v.name, v.data.into())).collect())
        .collect()
}

#[derive(PartialEq, Clone)]
pub enum SortOrder {
    ASC,
    DESC,
}

#[derive(Clone)]
pub struct Sort {
    pub field: String,
    pub order: SortOrder,
}

impl From<Sort> for service::Sort {
    fn from(value: Sort) -> Self {
        Self {
            field: value.field,
            order: match value.order {
                SortOrder::ASC => service::SortDir::ASC,
                SortOrder::DESC => service::SortDir::DESC,
            },
        }
    }
}

#[derive(Clone)]
pub enum SensorData {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Timestamp(chrono::NaiveDateTime),
    String(String),
    JSON(String),
}

impl From<SensorData> for service::SensorDataTypeValue {
    fn from(value: SensorData) -> Self {
        match value {
            SensorData::Int16(v) => service::SensorDataTypeValue::Int16(v),
            SensorData::Int32(v) => service::SensorDataTypeValue::Int32(v),
            SensorData::Int64(v) => service::SensorDataTypeValue::Int64(v),
            SensorData::Float32(v) => service::SensorDataTypeValue::Float32(v),
            SensorData::Float64(v) => service::SensorDataTypeValue::Float64(v),
            SensorData::Timestamp(v) => service::SensorDataTypeValue::Timestamp(v),
            SensorData::String(v) => service::SensorDataTypeValue::String(v),
            SensorData::JSON(v) => service::SensorDataTypeValue::JSON(v),
        }
    }
}

impl From<service::SensorDataTypeValue> for SensorData {
    fn from(value: service::SensorDataTypeValue) -> Self {
        match value {
            service::SensorDataTypeValue::Int16(v) => SensorData::Int16(v),
            service::SensorDataTypeValue::Int32(v) => SensorData::Int32(v),
            service::SensorDataTypeValue::Int64(v) => SensorData::Int64(v),
            service::SensorDataTypeValue::Float32(v) => SensorData::Float32(v),
            service::SensorDataTypeValue::Float64(v) => SensorData::Float64(v),
            service::SensorDataTypeValue::Timestamp(v) => SensorData::Timestamp(v),
            service::SensorDataTypeValue::String(v) => SensorData::String(v),
            service::SensorDataTypeValue::JSON(v) => SensorData::JSON(v),
        }
    }
}

pub struct DeviceEntry {
    pub id: i32,
    pub name: String,
}

impl From<service::DeviceInfo> for DeviceEntry {
    fn from(value: service::DeviceInfo) -> Self {
        Self {
            id: value.id.get_raw(),
            name: value.name,
        }
    }
}

pub struct SensorInfo {
    pub name: String,
    pub data: Vec<SensorDataInfo>,
}

impl From<service::SensorInfo> for SensorInfo {
    fn from(mut value: service::SensorInfo) -> Self {
        Self {
            name: value.name,
            data: value.data.drain(..).map(|v| v.into()).collect(),
        }
    }
}

pub struct SensorDataInfo {
    pub name: String,
    pub typ: SensorDataType,
}

impl From<service::SensorDataEntry> for SensorDataInfo {
    fn from(value: service::SensorDataEntry) -> Self {
        Self {
            name: value.name,
            typ: value.typ.into(),
        }
    }
}

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

impl From<service::SensorDataType> for SensorDataType {
    fn from(value: service::SensorDataType) -> Self {
        match value {
            service::SensorDataType::Int16 => SensorDataType::Int16,
            service::SensorDataType::Int32 => SensorDataType::Int32,
            service::SensorDataType::Int64 => SensorDataType::Int64,
            service::SensorDataType::Float32 => SensorDataType::Float32,
            service::SensorDataType::Float64 => SensorDataType::Float64,
            service::SensorDataType::Timestamp => SensorDataType::Timestamp,
            service::SensorDataType::String => SensorDataType::String,
            service::SensorDataType::JSON => SensorDataType::JSON,
        }
    }
}

pub struct MonitorConf {
    pub device_id: i32,
    pub sensor: String,
    pub typ: MonitorType,
    pub config: MonitorTypeConf,
}

impl From<MonitorConf> for service::MonitorConf {
    fn from(value: MonitorConf) -> Self {
        Self {
            id: 0,
            device_id: value.device_id,
            sensor: value.sensor,
            typ: value.typ.into(),
            config: Json(value.config.into()),
        }
    }
}

pub enum MonitorType {
    Log,
}

impl From<MonitorType> for service::MonitorType {
    fn from(value: MonitorType) -> Self {
        match value {
            MonitorType::Log => service::MonitorType::Log,
        }
    }
}

impl From<service::MonitorType> for MonitorType {
    fn from(value: service::MonitorType) -> Self {
        match value {
            service::MonitorType::Log => MonitorType::Log,
        }
    }
}

pub enum MonitorTypeConf {
    Log(MonitorLogConf),
}

impl From<MonitorTypeConf> for service::MonitorTypeConf {
    fn from(value: MonitorTypeConf) -> Self {
        match value {
            MonitorTypeConf::Log(v) => service::MonitorTypeConf::Log(v.into()),
        }
    }
}

impl From<service::MonitorTypeConf> for MonitorTypeConf {
    fn from(value: service::MonitorTypeConf) -> Self {
        match value {
            service::MonitorTypeConf::Log(v) => MonitorTypeConf::Log(v.into()),
        }
    }
}

pub struct MonitorLogConf {
    pub fields: Vec<String>,
    pub sort_field: String,
    pub sort_direction: SortDir,
    pub limit: i32,
}

impl From<MonitorLogConf> for service::MonitorLogConf {
    fn from(value: MonitorLogConf) -> Self {
        Self {
            fields: value.fields,
            sort_field: value.sort_field,
            sort_direction: value.sort_direction.into(),
            limit: value.limit,
        }
    }
}

impl From<service::MonitorLogConf> for MonitorLogConf {
    fn from(value: service::MonitorLogConf) -> Self {
        Self {
            fields: value.fields,
            sort_field: value.sort_field,
            sort_direction: value.sort_direction.into(),
            limit: value.limit,
        }
    }
}

pub enum SortDir {
    ASC,
    DESC,
}

impl From<SortDir> for service::SortDir {
    fn from(value: SortDir) -> Self {
        match value {
            SortDir::ASC => service::SortDir::ASC,
            SortDir::DESC => service::SortDir::DESC,
        }
    }
}

impl From<service::SortDir> for SortDir {
    fn from(value: service::SortDir) -> Self {
        match value {
            service::SortDir::ASC => SortDir::ASC,
            service::SortDir::DESC => SortDir::DESC,
        }
    }
}

pub struct MonitorConfListFilter {
    pub device_id: i32,
}

impl From<MonitorConfListFilter> for service::MonitorConfListFilter {
    fn from(value: MonitorConfListFilter) -> Self {
        Self {
            device_id: Some(value.device_id),
        }
    }
}

pub struct MonitorConfListEntry {
    pub id: i32,
    pub device_id: i32,
    pub sensor: String,
    pub typ: MonitorType,
    pub config: MonitorTypeConf,
}

impl From<service::MonitorConf> for MonitorConfListEntry {
    fn from(value: service::MonitorConf) -> Self {
        Self {
            id: value.id,
            device_id: value.device_id,
            sensor: value.sensor,
            typ: value.typ.into(),
            config: value.config.0.into(),
        }
    }
}

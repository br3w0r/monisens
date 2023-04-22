use std::collections::HashMap;

use crate::controller;
use actix_multipart::form::{bytes::Bytes, tempfile::TempFile, text::Text, MultipartForm};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, MultipartForm, ToSchema)]
pub struct TestUploadForm {
    #[schema(value_type = String, format = Binary)]
    #[multipart(rename = "file")]
    pub file: Bytes,
    #[schema(value_type = String, format = Byte)]
    pub name: Text<String>,
}

#[derive(Debug, MultipartForm, ToSchema)]
pub struct DeviceStartInitRequest {
    #[schema(value_type = String, format = Byte)]
    pub device_name: Text<String>,
    #[schema(value_type = String, format = Binary)]
    pub module_file: TempFile,
}

#[derive(Serialize, ToSchema)]
pub struct DeviceStartInitResponse {
    pub device_id: i32,
    pub conn_params: Vec<ConnParamConf>,
}

impl From<controller::DeviceInitData> for DeviceStartInitResponse {
    fn from(mut value: controller::DeviceInitData) -> Self {
        Self {
            device_id: value.id,
            conn_params: value.conn_params.drain(..).map(|v| v.into()).collect(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConnParamConf {
    pub name: String,
    pub typ: ConnParamType,
}

impl From<controller::ConnParamConf> for ConnParamConf {
    fn from(value: controller::ConnParamConf) -> Self {
        Self {
            name: value.name,
            typ: value.typ.into(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub enum ConnParamType {
    Bool,
    Int,
    Float,
    String,
}

impl From<controller::ConnParamType> for ConnParamType {
    fn from(value: controller::ConnParamType) -> Self {
        match value {
            controller::ConnParamType::Bool => ConnParamType::Bool,
            controller::ConnParamType::Int => ConnParamType::Int,
            controller::ConnParamType::Float => ConnParamType::Float,
            controller::ConnParamType::String => ConnParamType::String,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConnectDeviceRequest {
    pub device_id: i32,
    pub connect_conf: Vec<ConnParam>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConnParam {
    pub name: String,
    pub value: ConnParamValType,
}

impl From<ConnParam> for controller::ConnParam {
    fn from(value: ConnParam) -> Self {
        Self {
            name: value.name,
            value: value.value.into(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub enum ConnParamValType {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

impl From<ConnParamValType> for controller::ConnParamValType {
    fn from(value: ConnParamValType) -> Self {
        match value {
            ConnParamValType::Bool(v) => controller::ConnParamValType::Bool(v),
            ConnParamValType::Int(v) => controller::ConnParamValType::Int(v),
            ConnParamValType::Float(v) => controller::ConnParamValType::Float(v),
            ConnParamValType::String(v) => controller::ConnParamValType::String(v),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ObtainDeviceConfInfoRequest {
    pub device_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ObtainDeviceConfInfoResponse {
    pub device_conf_info: Vec<DeviceConfInfoEntry>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceConfInfoEntry {
    pub id: i32,
    pub name: String,
    pub data: DeviceConfInfoEntryType,
}

impl From<controller::DeviceConfInfoEntry> for DeviceConfInfoEntry {
    fn from(value: controller::DeviceConfInfoEntry) -> Self {
        Self {
            id: value.id,
            name: value.name,
            data: value.data.into(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub enum DeviceConfInfoEntryType {
    String(DeviceConfInfoEntryString),
    Int(DeviceConfInfoEntryInt),
    IntRange(DeviceConfInfoEntryIntRange),
    Float(DeviceConfInfoEntryFloat),
    FloatRange(DeviceConfInfoEntryFloatRange),
    JSON(DeviceConfInfoEntryJSON),
    ChoiceList(DeviceConfInfoEntryChoiceList),
    Section(Vec<DeviceConfInfoEntry>),
}

impl From<controller::DeviceConfInfoEntryType> for DeviceConfInfoEntryType {
    fn from(value: controller::DeviceConfInfoEntryType) -> Self {
        match value {
            controller::DeviceConfInfoEntryType::Section(mut v) => {
                DeviceConfInfoEntryType::Section(v.drain(..).map(|vv| vv.into()).collect())
            }
            controller::DeviceConfInfoEntryType::String(v) => {
                DeviceConfInfoEntryType::String(DeviceConfInfoEntryString {
                    required: v.required,
                    default: v.default,
                    min_len: v.min_len,
                    max_len: v.max_len,
                    match_regex: v.match_regex,
                })
            }
            controller::DeviceConfInfoEntryType::Int(v) => {
                DeviceConfInfoEntryType::Int(DeviceConfInfoEntryInt {
                    required: v.required,
                    default: v.default,
                    lt: v.lt,
                    gt: v.gt,
                    neq: v.neq,
                })
            }
            controller::DeviceConfInfoEntryType::IntRange(v) => {
                DeviceConfInfoEntryType::IntRange(DeviceConfInfoEntryIntRange {
                    required: v.required,
                    def_from: v.def_from,
                    def_to: v.def_to,
                    min: v.min,
                    max: v.max,
                })
            }
            controller::DeviceConfInfoEntryType::Float(v) => {
                DeviceConfInfoEntryType::Float(DeviceConfInfoEntryFloat {
                    required: v.required,
                    default: v.default,
                    lt: v.lt,
                    gt: v.gt,
                    neq: v.neq,
                })
            }
            controller::DeviceConfInfoEntryType::FloatRange(v) => {
                DeviceConfInfoEntryType::FloatRange(DeviceConfInfoEntryFloatRange {
                    required: v.required,
                    def_from: v.def_from,
                    def_to: v.def_to,
                    min: v.min,
                    max: v.max,
                })
            }
            controller::DeviceConfInfoEntryType::JSON(v) => {
                DeviceConfInfoEntryType::JSON(DeviceConfInfoEntryJSON {
                    required: v.required,
                    default: v.default,
                })
            }
            controller::DeviceConfInfoEntryType::ChoiceList(v) => {
                DeviceConfInfoEntryType::ChoiceList(DeviceConfInfoEntryChoiceList {
                    required: v.required,
                    default: v.default,
                    choices: v.choices,
                })
            }
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceConfInfoEntryString {
    pub required: bool,
    pub default: Option<String>,

    pub min_len: Option<i32>,
    pub max_len: Option<i32>,
    pub match_regex: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceConfInfoEntryInt {
    pub required: bool,
    pub default: Option<i32>,

    pub lt: Option<i32>,
    pub gt: Option<i32>,
    pub neq: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceConfInfoEntryIntRange {
    pub required: bool,
    pub def_from: Option<i32>,
    pub def_to: Option<i32>,

    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceConfInfoEntryFloat {
    pub required: bool,
    pub default: Option<f32>,

    pub lt: Option<f32>,
    pub gt: Option<f32>,
    pub neq: Option<f32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceConfInfoEntryFloatRange {
    pub required: bool,
    pub def_from: Option<f32>,
    pub def_to: Option<f32>,

    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceConfInfoEntryJSON {
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceConfInfoEntryChoiceList {
    pub required: bool,
    pub default: Option<i32>,

    pub choices: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConfigureDeviceRequest {
    pub device_id: i32,
    pub confs: Vec<DeviceConfEntry>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DeviceConfEntry {
    pub id: i32,
    pub data: Option<DeviceConfType>,
}

impl From<DeviceConfEntry> for controller::DeviceConfEntry {
    fn from(value: DeviceConfEntry) -> Self {
        controller::DeviceConfEntry {
            id: value.id,
            data: value.data.map(|v| v.into()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub enum DeviceConfType {
    String(String),
    Int(i32),
    IntRange([i32; 2]),
    Float(f32),
    FloatRange([f32; 2]),
    JSON(String),
    ChoiceList(i32),
}

impl From<DeviceConfType> for controller::DeviceConfType {
    fn from(value: DeviceConfType) -> Self {
        match value {
            DeviceConfType::String(v) => controller::DeviceConfType::String(v),
            DeviceConfType::Int(v) => controller::DeviceConfType::Int(v),
            DeviceConfType::IntRange(v) => controller::DeviceConfType::IntRange(v),
            DeviceConfType::Float(v) => controller::DeviceConfType::Float(v),
            DeviceConfType::FloatRange(v) => controller::DeviceConfType::FloatRange(v),
            DeviceConfType::JSON(v) => controller::DeviceConfType::JSON(v),
            DeviceConfType::ChoiceList(v) => controller::DeviceConfType::ChoiceList(v),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct InterruptDeviceInitRequest {
    pub device_id: i32,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct GetSensorDataRequest {
    #[validate(range(min = 1))]
    pub device_id: i32,
    #[validate(length(min = 1))]
    pub sensor: String,
    #[validate(length(min = 1))]
    pub fields: Vec<String>,
    #[validate]
    pub sort: Sort,
    pub from: Option<SensorData>,
    #[validate(range(max = 1000))]
    pub limit: Option<i32>,
}

impl From<GetSensorDataRequest> for controller::GetSensorDataPayload {
    fn from(value: GetSensorDataRequest) -> Self {
        Self {
            device_id: value.device_id,
            sensor: value.sensor,
            fields: value.fields,
            sort: value.sort.into(),
            from: value.from.map(|v| v.into()),
            limit: value.limit,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetSensorDataResponse {
    result: Vec<HashMap<String, SensorData>>,
}

impl From<controller::GetSensorDataResult> for GetSensorDataResponse {
    fn from(mut value: controller::GetSensorDataResult) -> Self {
        Self {
            result: value
                .drain(..)
                .map(|mut v| v.drain().map(|(field, val)| (field, val.into())).collect())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
pub enum SortOrder {
    ASC,
    DESC,
}

impl From<SortOrder> for controller::SortOrder {
    fn from(value: SortOrder) -> Self {
        match value {
            SortOrder::ASC => controller::SortOrder::ASC,
            SortOrder::DESC => controller::SortOrder::DESC,
        }
    }
}

#[derive(Clone, Debug, Validate, Deserialize, ToSchema)]
pub struct Sort {
    #[validate(length(min = 1))]
    pub field: String,
    pub order: SortOrder,
}

impl From<Sort> for controller::Sort {
    fn from(value: Sort) -> Self {
        Self {
            field: value.field,
            order: value.order.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub enum SensorData {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    #[schema(value_type = String)]
    Timestamp(chrono::NaiveDateTime),
    String(String),
    JSON(String),
}

impl From<SensorData> for controller::SensorData {
    fn from(value: SensorData) -> Self {
        match value {
            SensorData::Int16(v) => controller::SensorData::Int16(v),
            SensorData::Int32(v) => controller::SensorData::Int32(v),
            SensorData::Int64(v) => controller::SensorData::Int64(v),
            SensorData::Float32(v) => controller::SensorData::Float32(v),
            SensorData::Float64(v) => controller::SensorData::Float64(v),
            SensorData::Timestamp(v) => controller::SensorData::Timestamp(v),
            SensorData::String(v) => controller::SensorData::String(v),
            SensorData::JSON(v) => controller::SensorData::JSON(v),
        }
    }
}

impl From<controller::SensorData> for SensorData {
    fn from(value: controller::SensorData) -> Self {
        match value {
            controller::SensorData::Int16(v) => SensorData::Int16(v),
            controller::SensorData::Int32(v) => SensorData::Int32(v),
            controller::SensorData::Int64(v) => SensorData::Int64(v),
            controller::SensorData::Float32(v) => SensorData::Float32(v),
            controller::SensorData::Float64(v) => SensorData::Float64(v),
            controller::SensorData::Timestamp(v) => SensorData::Timestamp(v),
            controller::SensorData::String(v) => SensorData::String(v),
            controller::SensorData::JSON(v) => SensorData::JSON(v),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetDeviceListResponse {
    result: Vec<DeviceEntry>,
}

impl From<Vec<controller::DeviceEntry>> for GetDeviceListResponse {
    fn from(mut value: Vec<controller::DeviceEntry>) -> Self {
        Self {
            result: value.drain(..).map(|v| v.into()).collect(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceEntry {
    pub id: i32,
    pub name: String,
}

impl From<controller::DeviceEntry> for DeviceEntry {
    fn from(value: controller::DeviceEntry) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

#[derive(Clone, Debug, Validate, Deserialize, ToSchema)]
pub struct GetDeviceSensorInfoRequest {
    #[validate(range(min = 1))]
    pub device_id: i32,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct GetDeviceSensorInfoResponse {
    pub device_sensor_info: Vec<SensorInfo>,
}

impl From<Vec<controller::SensorInfo>> for GetDeviceSensorInfoResponse {
    fn from(mut value: Vec<controller::SensorInfo>) -> Self {
        let mut device_sensor_info: Vec<SensorInfo> = value.drain(..).map(|v| v.into()).collect();

        device_sensor_info.sort_unstable_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

        Self { device_sensor_info }
    }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SensorInfo {
    pub name: String,
    pub data: Vec<SensorDataInfo>,
}

impl From<controller::SensorInfo> for SensorInfo {
    fn from(mut value: controller::SensorInfo) -> Self {
        let mut data: Vec<SensorDataInfo> = value.data.drain(..).map(|v| v.into()).collect();

        data.sort_unstable_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

        Self {
            name: value.name,
            data: data,
        }
    }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SensorDataInfo {
    pub name: String,
    pub typ: SensorDataType,
}

impl From<controller::SensorDataInfo> for SensorDataInfo {
    fn from(value: controller::SensorDataInfo) -> Self {
        Self {
            name: value.name,
            typ: value.typ.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
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

impl From<controller::SensorDataType> for SensorDataType {
    fn from(value: controller::SensorDataType) -> Self {
        match value {
            controller::SensorDataType::Int16 => SensorDataType::Int16,
            controller::SensorDataType::Int32 => SensorDataType::Int32,
            controller::SensorDataType::Int64 => SensorDataType::Int64,
            controller::SensorDataType::Float32 => SensorDataType::Float32,
            controller::SensorDataType::Float64 => SensorDataType::Float64,
            controller::SensorDataType::Timestamp => SensorDataType::Timestamp,
            controller::SensorDataType::String => SensorDataType::String,
            controller::SensorDataType::JSON => SensorDataType::JSON,
        }
    }
}

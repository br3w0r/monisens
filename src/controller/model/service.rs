use std::{collections::HashMap, fmt, path::PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceInitState {
    Device,
    Sensors,
}

#[derive(Debug, Eq, Hash, PartialEq, Default, Clone, Copy)]
pub struct DeviceID(i32);

impl DeviceID {
    /// new creates a new DeviceID
    ///
    /// It must be used only by [IService](super::super::interface::service::IService)
    /// in [start_device_init](super::super::interface::service::IService::start_device_init)
    pub fn new(val: i32) -> Self {
        Self(val)
    }

    pub fn get_raw(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for DeviceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialOrd for DeviceID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[derive(Debug)]
pub struct DeviceInitData {
    pub id: DeviceID,
    pub module_dir: PathBuf,
    pub module_file: PathBuf,
    pub data_dir: PathBuf,
    pub full_data_dir: PathBuf,
    pub init_state: DeviceInitState,
}

#[derive(Default)]
pub struct SensorDataFilter {
    pub from: Option<(String, SensorDataTypeValue)>,
    pub to: Option<(String, SensorDataTypeValue)>,
    pub limit: Option<i32>,
    pub sort: Option<Sort>,
}

#[derive(Clone)]
pub struct Sort {
    pub field: String,
    pub order: SortDir,
}

pub type SensorDataList = Vec<SensorData>;

pub struct SensorData {
    pub name: String,
    pub data: SensorDataTypeValue,
}

#[derive(Debug, Clone)]
pub enum SensorDataTypeValue {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Timestamp(chrono::NaiveDateTime),
    String(String),
    JSON(String),
}

/// DeviceInfo contains basic info about device that may be read by user
pub struct DeviceInfo {
    pub id: DeviceID,
    pub display_name: String,
}

#[derive(Clone)]
pub struct SensorDataEntry {
    pub name: String,
    pub typ: SensorDataType,
}

pub struct Sensor {
    /// == sensor's table name // iss-96: this is not true. It's a human-readable name
    pub name: String,

    /// key in the [`HashMap`] is equal to [`SensorData`]`.name`
    pub data_map: HashMap<String, SensorDataEntry>,
}

/// SensorInfo contains basic info about device's sensors that may be read by user
pub struct SensorInfo {
    pub name: String,
    pub data: Vec<SensorDataEntry>,
}

#[derive(Clone)]
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

pub struct MonitorConf {
    pub id: i32,
    pub device_id: i32,
    pub sensor: String,
    pub typ: MonitorType,
    pub config: MonitorTypeConf,
}

pub enum MonitorType {
    Log,
    Line,
}

pub enum MonitorTypeConf {
    Log(MonitorLogConf),
    Line(MonitorLineConf),
}

pub struct MonitorLogConf {
    pub fields: Vec<String>,
    pub sort_field: String,
    pub sort_direction: SortDir,
    pub limit: i32,
}

pub struct MonitorLineConf {
    pub x_field: String,
    pub y_field: String,
    pub limit: i32,
}

pub struct MonitorConfListFilter {
    pub device_id: i32,
}

#[derive(Clone, PartialEq)]
pub enum SortDir {
    ASC,
    DESC,
}

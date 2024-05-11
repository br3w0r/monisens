#[derive(Debug)]
pub struct ConfInfoEntry {
    pub id: i32,
    pub name: String,
    pub data: ConfInfoEntryType,
}

pub type ConfInfo = Vec<ConfInfoEntry>;

#[derive(Debug)]
pub enum ConfInfoEntryType {
    Section(ConfInfo),
    String(ConfInfoEntryString),
    Int(ConfInfoEntryInt),
    IntRange(ConfInfoEntryIntRange),
    Float(ConfInfoEntryFloat),
    FloatRange(ConfInfoEntryFloatRange),
    JSON(ConfInfoEntryJSON),
    ChoiceList(ConfInfoEntryChoiceList),
}

#[derive(Debug)]
pub struct ConfInfoEntryString {
    pub required: bool,
    pub default: Option<String>,

    pub min_len: Option<i32>,
    pub max_len: Option<i32>,
    pub match_regex: Option<String>,
}

#[derive(Debug)]
pub struct ConfInfoEntryInt {
    pub required: bool,
    pub default: Option<i32>,

    pub lt: Option<i32>,
    pub gt: Option<i32>,
    pub neq: Option<i32>,
}

#[derive(Debug)]
pub struct ConfInfoEntryIntRange {
    pub required: bool,
    pub def_from: Option<i32>,
    pub def_to: Option<i32>,

    pub min: i32,
    pub max: i32,
}

#[derive(Debug)]
pub struct ConfInfoEntryFloat {
    pub required: bool,
    pub default: Option<f32>,

    pub lt: Option<f32>,
    pub gt: Option<f32>,
    pub neq: Option<f32>,
}

#[derive(Debug)]
pub struct ConfInfoEntryFloatRange {
    pub required: bool,
    pub def_from: Option<f32>,
    pub def_to: Option<f32>,

    pub min: f32,
    pub max: f32,
}

#[derive(Debug)]
pub struct ConfInfoEntryJSON {
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug)]
pub struct ConfInfoEntryChoiceList {
    pub required: bool,
    pub default: Option<i32>,

    pub choices: Vec<String>,
}

pub enum ConfType {
    String(String),
    Int(i32),
    IntRange([i32; 2]),
    Float(f32),
    FloatRange([f32; 2]),
    JSON(String),
    ChoiceList(i32),
}

pub struct ConfEntry {
    pub id: i32,
    pub data: Option<ConfType>,
}

#[derive(Debug)]
pub struct Message {
    pub msg: MessageType,
}

#[derive(Debug)]
pub enum MessageType {
    Sensor(SensorMsg),
    Common(CommonMsg),
}

#[derive(Debug)]
pub struct SensorMsg {
    pub name: String,
    pub data: Vec<SensorData>,
}

pub type SensorDataList = Vec<SensorData>;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct CommonMsg {
    pub code: MsgCode,
    pub msg: String,
}

#[derive(Debug)]
pub enum MsgCode {
    Info,
    Warn,
    Error,
}

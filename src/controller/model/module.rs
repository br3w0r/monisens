use std::ffi::CString;

use crate::module;

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

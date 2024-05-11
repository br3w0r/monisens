//! conv contains functions to convert:
//!   - between the controller and the bindings_gen types;
//!   - between C types and Rust types.

use libc::c_void;
use std::{
    ffi::{c_char, CStr, CString},
    ptr,
};

use super::bindings_gen as bg;
use crate::controller;

pub struct CStringHandle(Vec<*mut i8>);

impl CStringHandle {
    pub fn new() -> Self {
        CStringHandle(Vec::new())
    }

    pub fn save_and_return_str(&mut self, s: &str) -> *const c_char {
        self.0.push(CString::new(s).unwrap().into_raw());
        self.0.last().unwrap().clone()
    }

    pub fn save_and_return_cstring(&mut self, s: CString) -> *const c_char {
        self.0.push(CString::new(s).unwrap().into_raw());
        self.0.last().unwrap().clone()
    }
}

impl Drop for CStringHandle {
    fn drop(&mut self) {
        for ptr in self.0.iter() {
            unsafe {
                let _ = CString::from_raw(*ptr);
            }
        }
    }
}

pub fn device_conf_entry_vec_to_bg(
    confs: &Vec<controller::ConfEntry>,
    cstring_handle: &mut CStringHandle,
) -> Vec<bg::ConfEntry> {
    let mut confs_raw = Vec::with_capacity(confs.len());
    for conf in confs.iter() {
        confs_raw.push(bg::ConfEntry {
            id: conf.id,
            data: device_conf_option_to_ptr(&conf.data, cstring_handle),
        })
    }

    confs_raw
}

pub fn bg_device_conf_entry_vec_to_device_conf(confs: &Vec<bg::ConfEntry>) -> bg::Conf {
    bg::Conf {
        confs: confs.as_ptr() as _,
        confs_len: confs.len() as i32,
    }
}

pub fn bg_sensor_data_type_to_ctrl(val: &bg::SensorDataType) -> controller::SensorDataType {
    match val {
        bg::SensorDataType::SensorDataTypeInt16 => controller::SensorDataType::Int16,
        bg::SensorDataType::SensorDataTypeInt32 => controller::SensorDataType::Int32,
        bg::SensorDataType::SensorDataTypeInt64 => controller::SensorDataType::Int64,
        bg::SensorDataType::SensorDataTypeFloat32 => controller::SensorDataType::Float32,
        bg::SensorDataType::SensorDataTypeFloat64 => controller::SensorDataType::Float64,
        bg::SensorDataType::SensorDataTypeTimestamp => controller::SensorDataType::Timestamp,
        bg::SensorDataType::SensorDataTypeString => controller::SensorDataType::String,
        bg::SensorDataType::SensorDataTypeJSON => controller::SensorDataType::JSON,
    }
}

pub fn bg_message_to_ctrl(val: &bg::Message) -> controller::Message {
    match val.typ {
        bg::MessageType::MessageTypeSensor => controller::Message {
            msg: controller::MessageType::Sensor(bg_sensor_msg_to_ctrl(unsafe {
                &(*(val.data as *const bg::SensorMsg))
            })),
        },
        bg::MessageType::MessageTypeCommon => controller::Message {
            msg: controller::MessageType::Common(bg_common_msg_to_ctrl(unsafe {
                &(*(val.data as *const bg::CommonMsg))
            })),
        },
    }
}

pub fn bg_sensor_msg_to_ctrl(val: &bg::SensorMsg) -> controller::SensorMsg {
    let data_list = unsafe { std::slice::from_raw_parts(val.data, val.data_len as _) };

    controller::SensorMsg {
        name: str_from_c_char(val.name),
        data: data_list
            .into_iter()
            .map(|v| bg_sensor_data_msg_to_ctrl(v))
            .collect(),
    }
}

pub fn bg_sensor_data_msg_to_ctrl(val: &bg::SensorMsgData) -> controller::SensorData {
    let data = unsafe {
        match val.typ {
            bg::SensorDataType::SensorDataTypeInt16 => {
                controller::SensorDataTypeValue::Int16(*(val.data as *mut i16))
            }
            bg::SensorDataType::SensorDataTypeInt32 => {
                controller::SensorDataTypeValue::Int32(*(val.data as *mut i32))
            }
            bg::SensorDataType::SensorDataTypeInt64 => {
                controller::SensorDataTypeValue::Int64(*(val.data as *mut i64))
            }
            bg::SensorDataType::SensorDataTypeFloat32 => {
                controller::SensorDataTypeValue::Float32(*(val.data as *mut f32))
            }
            bg::SensorDataType::SensorDataTypeFloat64 => {
                controller::SensorDataTypeValue::Float64(*(val.data as *mut f64))
            }
            bg::SensorDataType::SensorDataTypeTimestamp => {
                let ts = chrono::DateTime::from_timestamp(*(val.data as *mut i64), 0).unwrap();
                controller::SensorDataTypeValue::Timestamp(ts.naive_utc())
            }
            bg::SensorDataType::SensorDataTypeString => {
                controller::SensorDataTypeValue::String(str_from_c_char(val.data as *mut c_char))
            }
            bg::SensorDataType::SensorDataTypeJSON => {
                controller::SensorDataTypeValue::JSON(str_from_c_char(val.data as *mut c_char))
            }
        }
    };

    controller::SensorData {
        name: str_from_c_char(val.name),
        data,
    }
}

pub fn bg_common_msg_to_ctrl(val: &bg::CommonMsg) -> controller::CommonMsg {
    controller::CommonMsg {
        code: bg_msg_code_to_ctrl(&val.code),
        msg: str_from_c_char(val.msg),
    }
}

pub fn bg_msg_code_to_ctrl(val: &bg::MsgCode) -> controller::MsgCode {
    match val {
        bg::MsgCode::MsgCodeInfo => controller::MsgCode::Info,
        bg::MsgCode::MsgCodeWarn => controller::MsgCode::Warn,
        bg::MsgCode::MsgCodeError => controller::MsgCode::Error,
    }
}

pub fn str_from_c_char(raw: *mut c_char) -> String {
    let cstr = unsafe { CStr::from_ptr(raw) };

    String::from_utf8_lossy(cstr.to_bytes()).to_string()
}

pub fn option_str_from_c_char(nullable_raw: *mut c_char) -> Option<String> {
    if nullable_raw.is_null() {
        None
    } else {
        Some(str_from_c_char(nullable_raw))
    }
}

// --------------------------------- private ------------------------------------

fn device_conf_option_to_ptr(
    data: &Option<controller::ConfType>,
    cstring_handle: &mut CStringHandle,
) -> *mut c_void {
    match data {
        Some(d) => device_conf_type_to_ptr(d, cstring_handle),
        None => ptr::null::<i32>() as _,
    }
}

fn device_conf_type_to_ptr(
    val: &controller::ConfType,
    cstring_handle: &mut CStringHandle,
) -> *mut c_void {
    match val {
        controller::ConfType::String(s) => cstring_handle.save_and_return_str(s) as _,
        controller::ConfType::Int(i) => i as *const i32 as _,
        controller::ConfType::IntRange(ir) => ir.as_ptr() as _,
        controller::ConfType::Float(f) => f as *const f32 as _,
        controller::ConfType::FloatRange(fr) => fr.as_ptr() as _,
        controller::ConfType::JSON(j) => cstring_handle.save_and_return_str(j) as _,
        controller::ConfType::ChoiceList(cl) => cl as *const i32 as _,
    }
}

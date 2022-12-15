use super::bindings_gen as bg;
use super::error::ModuleError;

use libc::c_void;
use std::ffi::CStr;

pub const VERSION: u8 = 1;

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
    typ: bg::ConnParamType,
}

pub type DeficeConfRec = Result<Vec<ConnParamConf>, ModuleError>;

fn device_connect_info(res: *mut DeficeConfRec, info: *const bg::DeviceConnectInfo) {
    if info.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("device_conf"));
        }
        return;
    }

    if unsafe { (*info).connection_params }.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("device_conf.connection_params"));
        }
        return;
    }

    let len = unsafe { (*info).connection_params_len as usize };
    let mut device_conf: Vec<ConnParamConf> = Vec::with_capacity(len);

    let s = unsafe { std::slice::from_raw_parts((*info).connection_params, len) };

    for i in s {
        if i.name.is_null() {
            unsafe {
                *res = Err(ModuleError::InvalidPointer(
                    "device_conf.connection_params[i].name",
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
                device_conf.push(ConnParamConf {
                    name: s.to_string(),
                    typ: i.typ.clone(),
                });
            }
        }
    }

    unsafe {
        *res = Ok(device_conf);
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

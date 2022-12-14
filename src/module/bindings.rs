use super::error::ModuleError;

use libc::{c_char, c_void};
use libloading::{self, Symbol};
use std::ffi::CStr;

pub const VERSION: u8 = 1;

pub type ModVersionFn = extern "C" fn() -> u8;

#[repr(C)]
#[derive(Clone, Debug)]
pub enum ConnParamType {
    Bool,
    Int,
    Float,
    String,
}

#[repr(C)]
pub struct CConnParamConf {
    name: *const c_char,
    typ: ConnParamType,
}

#[repr(C)]
pub struct CDeviceConf {
    connection_params: *const CConnParamConf,
    connection_params_len: i32,
}

#[derive(Debug)]
pub struct ConnParamConf {
    name: String,
    typ: ConnParamType,
}

pub type DeficeConfRec = Result<Vec<ConnParamConf>, ModuleError>;

fn device_conf_from_conf(res: &mut DeficeConfRec, conf: *const CDeviceConf) {
    if conf.is_null() {
        *res = Err(ModuleError::InvalidPointer("device_conf"));
        return;
    }

    if unsafe { (*conf).connection_params }.is_null() {
        *res = Err(ModuleError::InvalidPointer("device_conf.connection_params"));
        return;
    }

    let len = unsafe { (*conf).connection_params_len as usize };
    let mut device_conf: Vec<ConnParamConf> = Vec::with_capacity(len);

    let s = unsafe { std::slice::from_raw_parts((*conf).connection_params, len) };

    for i in s {
        if i.name.is_null() {
            *res = Err(ModuleError::InvalidPointer(
                "device_conf.connection_params[i].name",
            ));
            return;
        }

        match unsafe { CStr::from_ptr(i.name).to_str() } {
            Err(err) => {
                *res = Err(ModuleError::StrError(err.into()));
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

    *res = Ok(device_conf);
}

#[repr(C)]
pub struct CConnParam {
    pub name: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
pub struct CDeviceConnectInfo {
    pub connection_params: *const CConnParam,
    pub connection_params_len: i32,
}

pub struct Handle(*const c_void);

impl Handle {
    pub fn new() -> Self {
        Handle(std::ptr::null())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

pub type InitFn = unsafe extern "C" fn(*mut Handle);

pub type ObtainDeviceConfFn =
    unsafe extern "C" fn(*mut Handle, *mut DeficeConfRec, ObtainDeviceConfCallback);

pub type ObtainDeviceConfCallback = extern "C" fn(*mut DeficeConfRec, *const CDeviceConf);

pub extern "C" fn device_conf_callback(obj: *mut DeficeConfRec, conf: *const CDeviceConf) {
    let obj_ptr = unsafe { &mut *obj };
    device_conf_from_conf(obj_ptr, conf);
}

pub type ConnectDeviceFn =
    extern "C" fn(handle: *const Handle, connect_info: *const CDeviceConnectInfo) -> i32;

type DestroyFn = extern "C" fn(*const Handle);

#[repr(C)]
pub struct Functions {
    pub init: InitFn,
    pub obtain_device_conf: ObtainDeviceConfFn,
    pub destroy: DestroyFn,
    pub connect_device: ConnectDeviceFn,
}

pub type FunctionsFn = extern "C" fn() -> Functions;

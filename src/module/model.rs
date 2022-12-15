use super::bindings_gen as bg;
use super::error::{ComError, ModuleError};

use libc::c_void;
use std::ffi::{CStr, CString};

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

pub fn convert_com_error(err: u8) -> Result<(), ComError> {
    match err {
        0 => Ok(()),
        1 => Err(ComError::ConnectionError),
        2 => Err(ComError::InvalidArgument),
        _ => Err(ComError::Unknown),
    }
}

mod bindings;
mod bindings_gen;
pub mod error;
use std::ffi::CString;

use libc::c_void;
use libloading::{self, Symbol};
use std::error::Error;

use bindings::*;
use bindings_gen as bg;
use error::ModuleError;

use self::error::ComError;

pub struct Module {
    #[allow(dead_code)]
    lib: libloading::Library,
    handle: Handle,
    funcs: bg::Functions,
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            self.funcs.destroy.unwrap()(self.handle.handler());
        }
    }
}

impl Module {
    pub fn new(path: &str) -> Result<Module, Box<dyn Error>> {
        // TODO: unsafe {} where it's really unsafe
        unsafe {
            let lib = libloading::Library::new(path)?;

            // Check module version
            let mod_ver_fn: Symbol<bg::mod_version_fn> = lib.get(b"mod_version")?;

            let ver = mod_ver_fn.unwrap()();
            if ver != VERSION {
                return Err(ModuleError::InvalidVersion(ver, VERSION).into());
            }

            let funcs_fn: Symbol<bg::functions_fn> = lib.get(b"functions")?;
            let funcs = funcs_fn.unwrap()();

            let mut handler = Handle::new();
            funcs.init.unwrap()(handler.handler_ptr());

            if handler.is_null() {
                return Err(ModuleError::InvalidPointer("handle.0").into());
            }

            Ok(Module {
                lib,
                handle: handler,
                funcs,
            })
        }
    }

    pub fn obtain_device_info(&mut self) -> DeficeConfRec {
        let mut conf_rec: DeficeConfRec = Ok(Vec::new());
        unsafe {
            self.funcs.obtain_device_info.unwrap()(
                self.handle.handler(),
                &mut conf_rec as *mut DeficeConfRec as *mut c_void,
                Some(device_info_callback),
            )
        };

        conf_rec
    }

    pub fn connect_device(&mut self, conf: &mut DeviceConnectConf) -> Result<(), ComError> {
        let mut c_info = bg::DeviceConnectConf::from(conf);
        let err = unsafe {
            self.funcs.connect_device.unwrap()(
                &mut self.handle as *mut Handle as *mut c_void,
                &mut c_info as _,
            )
        };

        convert_com_error(err)
    }
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

fn convert_com_error(err: u8) -> Result<(), ComError> {
    match err {
        0 => Ok(()),
        1 => Err(ComError::ConnectionError),
        2 => Err(ComError::InvalidArgument),
        _ => Err(ComError::Unknown),
    }
}

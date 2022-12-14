mod bindings;
pub mod error;
use std::ffi::CString;

use libloading::{self, Symbol};
use std::error::Error;

use bindings::*;
use error::ModuleError;

use self::error::ComError;

pub struct Module {
    #[allow(dead_code)]
    lib: libloading::Library,
    handle: Handle,
    funcs: Functions,
}

impl Drop for Module {
    fn drop(&mut self) {
        (self.funcs.destroy)(&self.handle);
    }
}

impl Module {
    pub fn new(path: &str) -> Result<Module, Box<dyn Error>> {
        unsafe {
            let lib = libloading::Library::new(path)?;

            // Check module version
            let mod_ver_fn: Symbol<ModVersionFn> = lib.get(b"mod_version")?;
            let ver = mod_ver_fn();
            if ver != VERSION {
                return Err(ModuleError::InvalidVersion(ver, VERSION).into());
            }

            let funcs_fn: Symbol<FunctionsFn> = lib.get(b"functions")?;
            let funcs = funcs_fn();

            let mut handle = Handle::new();
            let handle_ptr = &mut handle as _;
            (funcs.init)(handle_ptr);

            if handle.is_null() {
                return Err(ModuleError::InvalidPointer("handle.0").into());
            }

            Ok(Module { lib, handle, funcs })
        }
    }

    pub fn obtain_device_conf(&mut self) -> DeficeConfRec {
        let mut conf_rec: DeficeConfRec = Ok(Vec::new());
        unsafe {
            (self.funcs.obtain_device_conf)(&mut self.handle, &mut conf_rec, device_conf_callback)
        };

        conf_rec
    }

    pub fn connect_device(&mut self, info: &mut DeviceConnectInfo) -> Result<(), ComError> {
        let c_info = CDeviceConnectInfo::from(info);
        let err = (self.funcs.connect_device)(&mut self.handle, &c_info);

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

pub struct DeviceConnectInfo {
    params: Vec<ConnParam>,
    c_params: Option<Vec<CConnParam>>,
}

impl DeviceConnectInfo {
    pub fn new(params: Vec<ConnParam>) -> Self {
        Self {
            params,
            c_params: None,
        }
    }
}

impl From<&mut DeviceConnectInfo> for CDeviceConnectInfo {
    fn from(info: &mut DeviceConnectInfo) -> Self {
        if let Some(ref c_params) = info.c_params {
            return CDeviceConnectInfo {
                connection_params: c_params.as_ptr(),
                connection_params_len: c_params.len() as i32,
            };
        }

        let mut c_params = Vec::with_capacity(info.params.len());

        for param in info.params.iter_mut() {
            c_params.push(CConnParam {
                name: param.get_c_name().as_ptr(),
                value: param.value.parsed().as_ptr(),
            });
        }

        info.c_params = Some(c_params);
        let c_params_ptr = info.c_params.as_ref().unwrap();

        CDeviceConnectInfo {
            connection_params: c_params_ptr.as_ptr(),
            connection_params_len: c_params_ptr.len() as i32,
        }
    }
}

fn convert_com_error(err: i32) -> Result<(), ComError> {
    match err {
        0 => Ok(()),
        1 => Err(ComError::ConnectionError),
        2 => Err(ComError::InvalidArgument),
        _ => Err(ComError::Unknown),
    }
}

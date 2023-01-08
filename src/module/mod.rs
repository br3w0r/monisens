mod bindings_gen;
pub mod error;
mod model;

use libc::c_void;
use libloading::{self, Symbol};
use std::error::Error;

use bindings_gen as bg;
use error::ModuleError;
use model::*;

use self::error::ComError;

pub use model::{ConnParam, ConnParamValType, DeviceConfInfo, DeviceConnectConf};

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
        let err =
            unsafe { self.funcs.connect_device.unwrap()(self.handle.handler(), &mut c_info as _) };

        convert_com_error(err)
    }

    pub fn obtain_device_conf_info(&mut self) -> DeviceConfInfoRec {
        let mut conf_rec: DeviceConfInfoRec = Ok(DeviceConfInfo::new(0));
        unsafe {
            self.funcs.obtain_device_conf_info.unwrap()(
                self.handle.handler(),
                &mut conf_rec as *mut DeviceConfInfoRec as *mut c_void,
                Some(device_conf_info_callback),
            )
        };

        conf_rec
    }
}

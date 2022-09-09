mod bindings;
pub mod error;

use libloading::{self, Symbol};
use std::error::Error;

use bindings::*;
use error::ModuleError;

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
}

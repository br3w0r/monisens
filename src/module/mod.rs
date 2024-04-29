mod bindings_gen;
pub mod error;
mod model;
mod conv;

use libc::c_void;
use libloading::{self, Symbol};
use std::{error::Error, ffi::CString, path::Path};

use bindings_gen as bg;
use model::*;

use crate::controller::interface::module::{IModule, IModuleFactory, MsgHandler};
use crate::controller;

pub use self::error::*;

pub struct Module {
    #[allow(dead_code)]
    lib: libloading::Library,
    handle: Handle,
    funcs: bg::Functions,

    msg_handle: Option<MsgHandle>,
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            self.funcs.destroy.unwrap()(self.handle.handler());
        }
    }
}

impl IModule for Module {
    fn obtain_device_conn_info(&mut self) -> Result<Vec<controller::ConnParamConf>, Box<dyn Error>> {
        let mut conf_rec: DeficeInfoRec = Ok(Vec::new());
        unsafe {
            self.funcs.obtain_device_info.unwrap()(
                self.handle.handler(),
                &mut conf_rec as *mut DeficeInfoRec as *mut c_void,
                Some(device_info_callback),
            )
        };

        let res = conf_rec?;

        Ok(res)
    }

    fn connect_device(&mut self, conf: controller::DeviceConnectConf) -> Result<(), Box<dyn Error>> {
        let mut c_string_handle = conv::CStringHandle::new();
        let conn_params = conv::conn_param_to_bg(&conf, &mut c_string_handle);
        let mut conf = conv::bg_conn_params_to_device_connect_conf(&conn_params);
        
        let err =
            unsafe { self.funcs.connect_device.unwrap()(self.handle.handler(), &mut conf as _) };

        convert_com_error(err)?;

        Ok(())
    }

    fn obtain_device_conf_info(&mut self) -> Result<controller::DeviceConfInfo, Box<dyn Error>> {
        let mut conf_rec: DeviceConfInfoRec = Ok(controller::DeviceConfInfo::new());
        unsafe {
            self.funcs.obtain_device_conf_info.unwrap()(
                self.handle.handler(),
                &mut conf_rec as *mut DeviceConfInfoRec as *mut c_void,
                Some(device_conf_info_callback),
            )
        };

        let res = conf_rec?;

        Ok(res)
    }

    fn configure_device(&mut self, confs: Vec<controller::DeviceConfEntry>) -> Result<(), Box<dyn Error>> {
        let mut c_string_handle = conv::CStringHandle::new();
        let confs_bg = conv::device_conf_entry_vec_to_bg(&confs, &mut c_string_handle);
        let mut device_conf_raw = conv::bg_device_conf_entry_vec_to_device_conf(&confs_bg);

        let err = unsafe {
            self.funcs.configure_device.unwrap()(self.handle.handler(), &mut device_conf_raw as _)
        };

        convert_com_error(err)?;

        Ok(())
    }

    fn obtain_sensor_type_infos(&mut self) -> Result<Vec<controller::Sensor>, Box<dyn Error>> {
        let mut infos_rec: SensorTypeInfosRec = Ok(vec![]);
        unsafe {
            self.funcs.obtain_sensor_type_infos.unwrap()(
                self.handle.handler(),
                &mut infos_rec as *mut SensorTypeInfosRec as *mut c_void,
                Some(sensor_type_infos_callback),
            )
        };

        let res = infos_rec?;

        Ok(res)
    }

    fn start<H: MsgHandler + 'static>(&mut self, msg_handler: H) -> Result<(), Box<dyn Error>> {
        self.msg_handle = Some(MsgHandle::new(msg_handler));

        unsafe {
            self.funcs.start.unwrap()(
                self.handle.handler(),
                self.msg_handle.as_ref().unwrap() as *const MsgHandle as *mut c_void,
                Some(handle_msg_callback),
            );
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        let err = unsafe { self.funcs.stop.unwrap()(self.handle.handler()) };

        convert_com_error(err)?;

        Ok(())
    }
}

impl IModuleFactory<Module> for Module {
    fn create_module<P: AsRef<Path>>(mod_path: P, data_dir: P) -> Result<Module, Box<dyn Error>> {
        // TODO: unsafe {} where it's really unsafe
        unsafe {
            let lib = libloading::Library::new(mod_path.as_ref().as_os_str())?;

            // Check module version
            let mod_ver_fn: Symbol<bg::mod_version_fn> = lib.get(b"mod_version")?;

            let ver = mod_ver_fn.unwrap()();
            if ver != VERSION {
                return Err(ModuleError::InvalidVersion(ver, VERSION).into());
            }

            let funcs_fn: Symbol<bg::functions_fn> = lib.get(b"functions")?;
            let funcs = funcs_fn.unwrap()();

            let mut handler = Handle::new();

            let data_dir_str = data_dir
                .as_ref()
                .to_str()
                .ok_or(Box::new(ModuleError::InvalidDataPath))?;

            #[cfg(unix)]
            let data_dir_str = data_dir_str.to_string() + "/";
            #[cfg(windows)]
            let data_dir_str = data_dir_str.to_string() + "\\";

            let data_dir_c = CString::new(data_dir_str)?;
            funcs.init.unwrap()(handler.handler_ptr(), data_dir_c.as_ptr() as _);

            if handler.is_null() {
                return Err(ModuleError::InvalidPointer("handle.0").into());
            }

            Ok(Module {
                lib,
                handle: handler,
                funcs,
                msg_handle: None,
            })
        }
    }
}

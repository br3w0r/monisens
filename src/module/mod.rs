mod bindings_gen;
mod conv;
pub mod error;
mod model;

use libc::c_void;
use libloading::{self, Symbol};
use std::{ffi::CString, path::Path};

use bindings_gen as bg;
use model::*;

use crate::controller;
use crate::controller::error::{CommonError, ErrorType};
use crate::controller::interface::module::{IModule, IModuleFactory, MsgHandler};

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
    fn obtain_device_conn_info(&mut self) -> Result<controller::ConfInfo, CommonError> {
        let mut conf_rec: ConfInfoRec = Ok(Vec::new());
        unsafe {
            self.funcs.obtain_device_conn_info.unwrap()(
                self.handle.handler(),
                &mut conf_rec as *mut ConfInfoRec as *mut c_void,
                Some(device_conn_info_callback),
            )
        };

        let res =
            conf_rec.map_err(|err| err.to_ctrl_error("failed to obtain device connection info"))?;

        Ok(res)
    }

    fn connect_device(&mut self, confs: Vec<controller::ConfEntry>) -> Result<(), CommonError> {
        let mut c_string_handle = conv::CStringHandle::new();
        let confs_bg = conv::device_conf_entry_vec_to_bg(&confs, &mut c_string_handle);
        let mut device_conf_raw = conv::bg_device_conf_entry_vec_to_device_conf(&confs_bg);

        let err = unsafe {
            self.funcs.connect_device.unwrap()(self.handle.handler(), &mut device_conf_raw as _)
        };

        convert_com_error(err).map_err(|err| err.to_ctrl_error("failed to connect to device"))?;

        Ok(())
    }

    fn obtain_device_conf_info(&mut self) -> Result<controller::ConfInfo, CommonError> {
        let mut conf_rec: ConfInfoRec = Ok(controller::ConfInfo::new());
        unsafe {
            self.funcs.obtain_device_conf_info.unwrap()(
                self.handle.handler(),
                &mut conf_rec as *mut ConfInfoRec as *mut c_void,
                Some(device_conf_info_callback),
            )
        };

        let res = conf_rec.map_err(|err| err.to_ctrl_error("failed to obtain device conf info"))?;

        Ok(res)
    }

    fn configure_device(&mut self, confs: Vec<controller::ConfEntry>) -> Result<(), CommonError> {
        let mut c_string_handle = conv::CStringHandle::new();
        let confs_bg = conv::device_conf_entry_vec_to_bg(&confs, &mut c_string_handle);
        let mut device_conf_raw = conv::bg_device_conf_entry_vec_to_device_conf(&confs_bg);

        let err = unsafe {
            self.funcs.configure_device.unwrap()(self.handle.handler(), &mut device_conf_raw as _)
        };

        convert_com_error(err).map_err(|err| err.to_ctrl_error("failed to configure device"))?;

        Ok(())
    }

    fn obtain_sensor_type_infos(&mut self) -> Result<Vec<controller::Sensor>, CommonError> {
        let mut infos_rec: SensorTypeInfosRec = Ok(vec![]);
        unsafe {
            self.funcs.obtain_sensor_type_infos.unwrap()(
                self.handle.handler(),
                &mut infos_rec as *mut SensorTypeInfosRec as *mut c_void,
                Some(sensor_type_infos_callback),
            )
        };

        let res =
            infos_rec.map_err(|err| err.to_ctrl_error("failed to obtain sensor type infos"))?;

        Ok(res)
    }

    fn start<H: MsgHandler + 'static>(&mut self, msg_handler: H) -> Result<(), CommonError> {
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

    fn stop(&mut self) -> Result<(), CommonError> {
        let err = unsafe { self.funcs.stop.unwrap()(self.handle.handler()) };

        convert_com_error(err).map_err(|err| err.to_ctrl_error("failed to stop module"))?;

        Ok(())
    }
}

impl IModuleFactory<Module> for Module {
    fn create_module<P: AsRef<Path>>(mod_path: P, data_dir: P) -> Result<Module, CommonError> {
        // TODO: unsafe {} where it's really unsafe
        unsafe {
            let lib = libloading::Library::new(mod_path.as_ref().as_os_str()).map_err(|err| {
                CommonError::new(ErrorType::IO, format!("failed to load dynamic library"))
                    .with_source(err)
            })?;

            // Check module version
            let mod_ver_fn: Symbol<bg::mod_version_fn> =
                lib.get(b"mod_version").map_err(|err| {
                    CommonError::new(
                        ErrorType::IO,
                        format!("failed to call 'mod_version' function"),
                    )
                    .with_source(err)
                })?;

            let ver = mod_ver_fn.unwrap()();
            if ver != VERSION {
                return Err(ModuleError::InvalidVersion(ver, VERSION)
                    .to_ctrl_error("the dynamic library has invalid version"));
            }

            let funcs_fn: Symbol<bg::functions_fn> = lib.get(b"functions").map_err(|err| {
                CommonError::new(
                    ErrorType::IO,
                    format!("failed to call 'functions' function"),
                )
                .with_source(err)
            })?;
            let funcs = funcs_fn.unwrap()();

            validate_funcs_present(&funcs).map_err(|func_name| {
                CommonError::new(
                    ErrorType::IO,
                    format!("function '{}' is not present in the module", func_name),
                )
            })?;

            let mut handler = Handle::new();

            let data_dir_str = data_dir
                .as_ref()
                .to_str()
                .ok_or(Box::new(ModuleError::InvalidDataPath))
                .map_err(|err| err.to_ctrl_error("failed to convert data path to string"))?;

            #[cfg(unix)]
            let data_dir_str = data_dir_str.to_string() + "/";
            #[cfg(windows)]
            let data_dir_str = data_dir_str.to_string() + "\\";

            let data_dir_c = CString::new(data_dir_str).map_err(|err| {
                CommonError::new(
                    ErrorType::Internal,
                    format!("failed to convert data path to CString"),
                )
                .with_source(err)
            })?;
            funcs.init.unwrap()(handler.handler_ptr(), data_dir_c.as_ptr() as _);

            if handler.is_null() {
                return Err(ModuleError::InvalidPointer("handle.0")
                    .to_ctrl_error("failed to init module (handler is null)"));
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

fn validate_funcs_present(funcs: &bg::Functions) -> Result<(), &str> {
    if funcs.init.is_none() {
        return Err("init");
    }
    if funcs.destroy.is_none() {
        return Err("destroy");
    }
    if funcs.obtain_device_conn_info.is_none() {
        return Err("obtain_device_conn_info");
    }
    if funcs.connect_device.is_none() {
        return Err("connect_device");
    }
    if funcs.obtain_device_conf_info.is_none() {
        return Err("obtain_device_conf_info");
    }
    if funcs.configure_device.is_none() {
        return Err("configure_device");
    }
    if funcs.obtain_sensor_type_infos.is_none() {
        return Err("obtain_sensor_type_infos");
    }
    if funcs.start.is_none() {
        return Err("start");
    }
    if funcs.stop.is_none() {
        return Err("stop");
    }

    Ok(())
}

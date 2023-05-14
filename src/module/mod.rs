mod bindings_gen;
pub mod error;
mod model;

use libc::c_void;
use libloading::{self, Symbol};
use std::{error::Error, ffi::CString};

use bindings_gen as bg;
use model::*;

pub use self::error::*;

pub use model::{
    CommonMsg, ConnParam, ConnParamEntryInfo, ConnParamInfo, ConnParamType, ConnParamValType,
    DeviceConfEntry, DeviceConfInfo, DeviceConfInfoEntry, DeviceConfInfoEntryType, DeviceConfType,
    DeviceConnectConf, Message, MessageType, MsgCode, MsgHandler, SensorDataType, SensorMsg,
    SensorMsgData, SensorMsgDataType, SensorTypeInfo,
};

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

impl Module {
    pub fn new(mod_path: &str, data_dir: &str) -> Result<Module, Box<dyn Error>> {
        // TODO: unsafe {} where it's really unsafe
        unsafe {
            let lib = libloading::Library::new(mod_path)?;

            // Check module version
            let mod_ver_fn: Symbol<bg::mod_version_fn> = lib.get(b"mod_version")?;

            let ver = mod_ver_fn.unwrap()();
            if ver != VERSION {
                return Err(ModuleError::InvalidVersion(ver, VERSION).into());
            }

            let funcs_fn: Symbol<bg::functions_fn> = lib.get(b"functions")?;
            let funcs = funcs_fn.unwrap()();

            let mut handler = Handle::new();

            let data_dir_c = CString::new(data_dir)?;
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

    pub fn obtain_device_info(&mut self) -> DeficeInfoRec {
        let mut conf_rec: DeficeInfoRec = Ok(Vec::new());
        unsafe {
            self.funcs.obtain_device_info.unwrap()(
                self.handle.handler(),
                &mut conf_rec as *mut DeficeInfoRec as *mut c_void,
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

    /// `Module::configure_device` sends config to the module.
    ///
    /// `confs` **must**
    /// - include all entries with ids returned from `Module::obtain_device_conf_info`
    /// - pass validation based on info from `Module::obtain_device_conf_info`
    ///
    /// Thus, `Module::obtain_device_conf_info` **must** be called before `Module::configure_device`
    pub fn configure_device(&mut self, confs: &mut Vec<DeviceConfEntry>) -> Result<(), ComError> {
        // TODO: validate confs (issue #60)
        let confs_raw = build_device_conf_entry_raw_vec(confs);
        let mut device_conf_raw = build_device_conf(&confs_raw);

        let err = unsafe {
            self.funcs.configure_device.unwrap()(self.handle.handler(), &mut device_conf_raw as _)
        };

        convert_com_error(err)
    }

    pub fn obtain_sensor_type_infos(&mut self) -> SensorTypeInfosRec {
        let mut infos_rec: SensorTypeInfosRec = Ok(vec![]);
        unsafe {
            self.funcs.obtain_sensor_type_infos.unwrap()(
                self.handle.handler(),
                &mut infos_rec as *mut SensorTypeInfosRec as *mut c_void,
                Some(sensor_type_infos_callback),
            )
        };

        infos_rec
    }

    pub fn start<H: MsgHandler + 'static>(&mut self, msg_handler: H) -> Result<(), ComError> {
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

    pub fn stop(&mut self) -> Result<(), ComError> {
        let err = unsafe { self.funcs.stop.unwrap()(self.handle.handler()) };

        convert_com_error(err)
    }
}

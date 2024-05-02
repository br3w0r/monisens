use crate::controller::error::CommonError;
use std::path::Path;

use super::super::model;

pub trait IModule {
    fn obtain_device_conn_info(&mut self) -> Result<Vec<model::ConnParamConf>, CommonError>;
    fn connect_device(&mut self, conf: model::DeviceConnectConf) -> Result<(), CommonError>;
    fn obtain_device_conf_info(&mut self) -> Result<model::DeviceConfInfo, CommonError>;
    fn configure_device(&mut self, confs: Vec<model::DeviceConfEntry>) -> Result<(), CommonError>;
    fn obtain_sensor_type_infos(&mut self) -> Result<Vec<model::Sensor>, CommonError>;
    fn start<H: MsgHandler + 'static>(&mut self, msg_handler: H) -> Result<(), CommonError>;
    fn stop(&mut self) -> Result<(), CommonError>;
}

pub trait IModuleFactory<M: IModule> {
    fn create_module<P: AsRef<Path>>(mod_path: P, data_dir: P) -> Result<M, CommonError>;
}

pub trait MsgHandler: Send + Sync {
    fn handle_msg(&self, msg: model::Message);
}

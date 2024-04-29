use std::{error::Error, path::Path};
use super::super::model;

pub trait IModule {
    // fn new<P: AsRef<Path>>(mod_path: P, data_dir: P) -> Result<M, Box<dyn Error>>;
    fn obtain_device_conn_info(&mut self) -> Result<Vec<model::ConnParamConf>, Box<dyn Error>>;
    fn connect_device(&mut self, conf: model::DeviceConnectConf) -> Result<(), Box<dyn Error>>;
    fn obtain_device_conf_info(&mut self) -> Result<model::DeviceConfInfo, Box<dyn Error>>;
    fn configure_device(&mut self, confs: Vec<model::DeviceConfEntry>) -> Result<(), Box<dyn Error>>;
    fn obtain_sensor_type_infos(&mut self) -> Result<Vec<model::Sensor>, Box<dyn Error>>;
    fn start<H: MsgHandler + 'static>(&mut self, msg_handler: H) -> Result<(), Box<dyn Error>>;
    fn stop(&mut self) -> Result<(), Box<dyn Error>>;
}

pub trait IModuleFactory<M: IModule> {
    fn create_module<P: AsRef<Path>>(mod_path: P, data_dir: P) -> Result<M, Box<dyn Error>>;
}

pub trait MsgHandler: Send + Sync {
    fn handle_msg(&self, msg: model::Message);
}

use super::super::interface::service;
use super::super::msg;
use crate::module;

// TODO: issue #81
// pub enum DeviceState {
//     Inited,
//     Connected,
//     Configurated,
// }

pub struct Device<S: service::IService> {
    pub id: super::DeviceID,
    pub module: module::Module,
    pub msg_handler: Option<msg::Handler<S>>,
    // TODO: issue #81
    // pub state: DeviceState,
}

use super::super::interface::{service::IService, module::IModule};
use super::super::msg;

// TODO: issue #81
// pub enum DeviceState {
//     Inited,
//     Connected,
//     Configurated,
// }

pub struct Device<S: IService, M: IModule> {
    pub id: super::DeviceID,
    pub module: M,
    pub msg_handler: Option<msg::Handler<S>>,
    // TODO: issue #81
    // pub state: DeviceState,
}

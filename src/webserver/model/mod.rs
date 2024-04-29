pub mod contract;

use super::config;
use crate::controller::Controller;
use crate::service::Service;
use crate::module::Module;

pub struct ServiceState {
    pub ctrl: Controller<Service, Module, Module>,
}

pub struct AppState {
    pub conf: config::AppConfig,
}

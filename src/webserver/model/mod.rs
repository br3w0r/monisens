pub mod contract;

use super::config;
use crate::controller::Controller;
use crate::module::Module;
use crate::service::Service;

pub struct ServiceState {
    pub ctrl: Controller<Service, Module, Module>,
}

pub struct AppState {
    pub conf: config::AppConfig,
}

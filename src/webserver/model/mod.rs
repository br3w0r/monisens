pub mod contract;

use super::config;
use crate::controller::Controller;
use crate::service::Service;

pub struct ServiceState {
    pub ctrl: Controller<Service>,
}

pub struct AppState {
    pub conf: config::AppConfig,
}

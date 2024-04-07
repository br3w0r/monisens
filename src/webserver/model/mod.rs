pub mod contract;

use crate::controller;
use super::config;

pub struct ServiceState {
    pub ctrl: controller::Controller,
}

pub struct AppState {
    pub conf: config::AppConfig,
}

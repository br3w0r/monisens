use crate::debug_from_display;
use thiserror::Error;

use std::fmt;

use crate::tool::validation::ValidationError;

use super::device;

#[derive(Error)]
pub enum ServiceError {
    #[error("device sensor vaildation failed: {0}")]
    DeviceSensorInitErr(String),
    #[error("failed to validate name '{0}': {1}")]
    NameValidationErr(String, ValidationError),
    #[error("device '{0}' has already been initialized")]
    DeviceAlreadyInitialized(device::DeviceID),
    #[error("path is invalid")]
    InvalidPath,
}

debug_from_display!(ServiceError);

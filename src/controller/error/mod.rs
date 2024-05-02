mod common;

use thiserror::Error;

pub use common::*;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("unknown device with id {0}")]
    UnknownDevice(i32),
    // TODO: issue #81
    // #[error("device already connected")]
    // DeviceAlreadyConnected,
    // #[error("device is not connected")]
    // DeviceNotConnected
    #[error("incorrect payload was given to method: {0}")]
    IncorrectPayload(String),

    #[error("common error: {0}")]
    CommonError(#[source] CommonError),

    #[error("other error: {0}")]
    Other(#[source] Box<dyn std::error::Error + 'static>),
}

impl From<CommonError> for ControllerError {
    fn from(e: CommonError) -> Self {
        Self::CommonError(e)
    }
}

impl From<Box<dyn std::error::Error + 'static>> for ControllerError {
    fn from(e: Box<dyn std::error::Error + 'static>) -> Self {
        Self::Other(e)
    }
}

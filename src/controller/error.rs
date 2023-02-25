use std::fmt;
use thiserror::Error;

use crate::debug_from_display;

#[derive(Error)]
pub enum ControllerError {
    #[error("unknown device with id {0}")]
    UnknownDevice(i32),
    // TODO: issue #81
    // #[error("device already connected")]
    // DeviceAlreadyConnected,
    // #[error("device is not connected")]
    // DeviceNotConnected
}

debug_from_display!(ControllerError);

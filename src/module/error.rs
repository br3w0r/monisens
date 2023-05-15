use std::fmt;
use thiserror::Error;

use crate::debug_from_display;

#[derive(Error)]
pub enum ModuleError {
    #[error("lib version '{0}' doesn't match current supported version '{1}'")]
    InvalidVersion(
        u8, /* lib version */
        u8, /* current supported version */
    ),
    #[error("invalid pointer to variable or field '{0}'")]
    InvalidPointer(&'static str),
    #[error("failed to convert pointer to char into string")]
    StrError(Box<dyn std::error::Error>),
    #[error("data path is invalid")]
    InvalidDataPath,
}

debug_from_display!(ModuleError);

// ComError describes recoverable errors that
// are not neccessarily related to a broken library.
// It may be connection error or parameter error.
#[derive(Error)]
pub enum ComError {
    #[error("unknown ComError type")]
    Unknown,
    #[error("ConnectionError: failed to communicate with remote device")]
    ConnectionError,
    #[error("InvalidArgument: some of parameters are wrong")]
    InvalidArgument,
}

debug_from_display!(ComError);

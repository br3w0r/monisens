use thiserror::Error;

use crate::controller::error::{CommonError, ErrorType};

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
    // TODO: Find out why it's not used
    #[error("failed to convert pointer to char into string")]
    StrError(Box<dyn std::error::Error>),
    #[error("data path is invalid")]
    InvalidDataPath,
}

impl ModuleError {
    pub fn to_ctrl_type(&self) -> ErrorType {
        match self {
            ModuleError::InvalidVersion(_, _) => ErrorType::InvalidInput,
            ModuleError::InvalidPointer(_) => ErrorType::Internal,
            ModuleError::StrError(_) => ErrorType::Internal,
            ModuleError::InvalidDataPath => ErrorType::Internal,
        }
    }

    pub fn to_ctrl_error<S: Into<String>>(self, msg: S) -> CommonError {
        CommonError::new(self.to_ctrl_type(), msg.into()).with_source(self)
    }
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

impl ComError {
    pub fn to_ctrl_type(&self) -> ErrorType {
        match self {
            ComError::Unknown => ErrorType::Unknown,
            ComError::ConnectionError => ErrorType::IO,
            ComError::InvalidArgument => ErrorType::InvalidInput,
        }
    }

    pub fn to_ctrl_error<S: Into<String>>(self, msg: S) -> CommonError {
        CommonError::new(self.to_ctrl_type(), msg.into()).with_source(self)
    }
}

debug_from_display!(ComError);

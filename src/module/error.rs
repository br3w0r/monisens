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
}

debug_from_display!(ModuleError);

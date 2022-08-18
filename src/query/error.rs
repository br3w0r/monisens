use std::fmt;
use thiserror::Error;

use crate::debug_from_display;

#[derive(Error)]
pub enum BuilderError {
    #[error("value to get or insert is not of type 'Vec'")]
    NotVec,
    #[error("value to get or insert is not of type 'dyn Any'")]
    NotAny,
}

debug_from_display!(BuilderError);

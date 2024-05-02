use thiserror::Error;

use super::{FieldOption, FieldType};
use crate::debug_from_display;
use crate::tool::validation::ValidationError;

#[derive(Error)]
pub enum TableError {
    #[error("failed to validate field '{0}': {1}")]
    Field(String, FieldError),
    #[error("validation failed: {0}")]
    Validation(ValidationError),
}

debug_from_display!(TableError);

#[derive(Error)]
pub enum FieldError {
    #[error("failed to assign a duplicate of option: {0:?}")]
    DuplicateOption(FieldOption),
    #[error("validation failed: {0}")]
    Validation(ValidationError),
    #[error("failed to assign invalid option '{0:?}' for field of type '{1:?}'")]
    InvalidTypeOption(FieldType, FieldOption),
}

debug_from_display!(FieldError);

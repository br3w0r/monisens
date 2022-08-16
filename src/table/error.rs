use std::error::Error;
use std::fmt;

use super::{FieldOption, FieldType};
use crate::tool::validation::ValidationError;

pub enum TableError {
    Field(String, FieldError),
    Validation(ValidationError),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableError::Field(field, err) => {
                write!(f, "failed to validate field '{}': {}", field, err)
            }
            TableError::Validation(err) => write!(f, "validation failed: {}", err),
        }
    }
}

impl fmt::Debug for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f, "{}", self)
    }
}

impl Error for TableError{}

pub enum FieldError {
    DuplicateOption(FieldOption),
    Validation(ValidationError),
    InvalidTypeOption(FieldType, FieldOption),
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldError::DuplicateOption(opt) => {
                write!(f, "failed to assign a duplicate of option: {:?}", opt)
            }
            FieldError::Validation(err) => write!(f, "validation failed: {}", err),
            FieldError::InvalidTypeOption(typ, opt) => write!(
                f,
                "failed to assign invalid option '{:?}' for field of type '{:?}'",
                opt, typ
            ),
        }
    }
}

impl fmt::Debug for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       write!(f, "{}", self)
    }
}

impl Error for FieldError{}

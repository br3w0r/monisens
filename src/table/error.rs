// use std::error::Error;

use crate::tool::validation::ValidationError;
use super::{FieldType, FieldOption};

// TODO: человекочитаемые ошибки

#[derive(Debug)]
pub enum TableError {
    Field((String, FieldError)),
    Validation(ValidationError),
}

// impl Error for TableError{}

#[derive(Debug)]
pub enum FieldError {
    DuplicateOption(FieldOption),
    Validation(ValidationError),
    InvalidTypeOption(FieldType, FieldOption),
}

// impl Error for FieldError{}

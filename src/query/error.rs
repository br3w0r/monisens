use std::fmt;
use thiserror::Error;

use crate::debug_from_display;

#[derive(Error)]
pub enum BuilderError {
    #[error("value saved for the key is not of type 'Vec'")]
    NotVec,
    #[error("value saved for the key is not of type 'dyn Any'")]
    NotAny,
}

debug_from_display!(BuilderError);

#[derive(Error)]
pub enum SelectError {
    #[error("select statements must have at least one result column")]
    NoColumns,
}

debug_from_display!(SelectError);

#[derive(Error)]
pub enum InsertError {
    #[error("insert statements must specify a table")]
    NoTable,
    #[error("insert statements must have at least one set of values")]
    NoValues,
}

debug_from_display!(InsertError);

#[derive(Error)]
pub enum ValuesError {
    #[error("no values where given")]
    NoValues,
}

debug_from_display!(ValuesError);

#[derive(Error)]
pub enum DeleteError {
    #[error("delete statements must specify a table")]
    NoTable,
}

debug_from_display!(DeleteError);

#[derive(Error)]
pub enum ExprError {
    #[error("no args for expression")]
    NoArgs
}

debug_from_display!(ExprError);

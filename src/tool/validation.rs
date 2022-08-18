use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use crate::debug_from_display;
use thiserror::Error;

lazy_static! {
    static ref RE_SINGLE_WORD: Regex = Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
}

#[derive(Error)]
pub enum ValidationError {
    #[error("name contains unsupported chars")]
    UnsupportedChars,
}

debug_from_display!(ValidationError);

pub fn validate_word(s: &str) -> Result<(), ValidationError> {
    if RE_SINGLE_WORD.is_match(s) {
        return Ok(());
    }

    Err(ValidationError::UnsupportedChars)
}

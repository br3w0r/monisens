use crate::debug_from_display;
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

lazy_static! {
    static ref RE_SINGLE_WORD: Regex = Regex::new(r"^[a-zA-Z0-9_\-]+$").unwrap();
    static ref RE_MULTIPLE_WORDS: Regex = Regex::new(r"^[a-zA-Z0-9_\- ]+$").unwrap();
    static ref RE_SNAKE_CASE: Regex = Regex::new(r"^[a-zA-Z0-9]+(_[a-zA-Z0-9]+)*$").unwrap();
}

#[derive(Error)]
pub enum ValidationError {
    #[error("word's len is bigger than {0}")]
    LengthExceeded(usize),
    #[error("word contains unsupported chars")]
    UnsupportedChars,
    #[error("word is not in snake_case")]
    NotSnakeCase,
}

debug_from_display!(ValidationError);

pub fn validate_len(s: &str, max_len: usize) -> Result<(), ValidationError> {
    if s.len() > max_len {
        return Err(ValidationError::LengthExceeded(max_len));
    }

    Ok(())
}

pub fn validate_chars(s: &str) -> Result<(), ValidationError> {
    if RE_SINGLE_WORD.is_match(s) {
        return Ok(());
    }

    Err(ValidationError::UnsupportedChars)
}

pub fn validate_multiple_words(s: &str) -> Result<(), ValidationError> {
    if RE_MULTIPLE_WORDS.is_match(s) {
        return Ok(());
    }

    Err(ValidationError::UnsupportedChars)
}

pub fn validate_snake_case(s: &str) -> Result<(), ValidationError> {
    if RE_SNAKE_CASE.is_match(s) {
        return Ok(());
    }

    Err(ValidationError::NotSnakeCase)
}

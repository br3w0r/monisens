use std::fmt;
use std::error::Error;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_SINGLE_WORD: Regex = Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
}

#[derive(Debug)]
pub enum ValidationError {
    UnsupportedChars
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::UnsupportedChars => write!(f, "name contains unsupported chars")
        }
    }
}

impl Error for ValidationError {}

pub fn validate_word(s: &str) -> Result<(), ValidationError> {
    if RE_SINGLE_WORD.is_match(s) {
        return Ok(());
    }

    Err(ValidationError::UnsupportedChars)
}

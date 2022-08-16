use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::fmt;

lazy_static! {
    static ref RE_SINGLE_WORD: Regex = Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
}

pub enum ValidationError {
    UnsupportedChars,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::UnsupportedChars => write!(f, "name contains unsupported chars"),
        }
    }
}

impl fmt::Debug for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self as &dyn fmt::Display).fmt(f)
    }
}

impl Error for ValidationError {}

pub fn validate_word(s: &str) -> Result<(), ValidationError> {
    if RE_SINGLE_WORD.is_match(s) {
        return Ok(());
    }

    Err(ValidationError::UnsupportedChars)
}

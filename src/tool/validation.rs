use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_SINGLE_WORD: Regex = Regex::new(r"^[A-Za-z0-9_]+$").unwrap();
}

#[derive(Debug)]
pub enum ValidationError {
    UnsupportedChars
}

pub fn validate_word(s: &str) -> Option<ValidationError> {
    if RE_SINGLE_WORD.is_match(s) {
        return None;
    }

    Some(ValidationError::UnsupportedChars)
}

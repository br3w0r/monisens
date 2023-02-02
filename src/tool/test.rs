#[cfg(test)]
use super::validation::validate_chars;

#[test]
fn test_validate_word() {
    // Success
    let res = validate_chars("one_word");
    assert!(!res.is_err());

    let res = validate_chars("numbers1234567890");
    assert!(!res.is_err());

    // Failure
    let res = validate_chars("two words");
    assert!(res.is_err());

    let res = validate_chars("?unknown_chars");
    assert!(res.is_err());
}

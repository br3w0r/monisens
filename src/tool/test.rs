#[cfg(test)]
use super::validation::validate_word;

#[test]
fn test_validate_word() {
    // Success
    let res = validate_word("one_word");
    assert!(!res.is_err());

    let res = validate_word("numbers1234567890");
    assert!(!res.is_err());

    // Failure
    let res = validate_word("two words");
    assert!(res.is_err());

    let res = validate_word("?unknown_chars");
    assert!(res.is_err());
}

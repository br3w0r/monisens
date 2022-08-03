#[cfg(test)]
use super::{Field, FieldError, FieldOption, FieldType, Table};
use crate::tool::validation::ValidationError;

// Test that `Field`'s capacity is being calculated properly
#[test]
fn field_capacity() {
    let mut f = Field::new(1, "test_field".to_string(), FieldType::Int64).unwrap();
    f.add_opt(FieldOption::PrimaryKey).unwrap();
    f.add_opt(FieldOption::NotNull).unwrap();
    f.add_opt(FieldOption::Unique).unwrap();
    f.add_opt(FieldOption::AutoIncrement).unwrap();

    let calc_size = f.parse_size();
    let parsed = f.parse().unwrap();

    assert_eq!(
        parsed.len(),
        parsed.capacity(),
        "len: {}, cap: {}",
        parsed.len(),
        parsed.capacity()
    );

    assert_eq!(
        calc_size,
        parsed.capacity(),
        "calc_size: {}, parsed.capacity(): {}",
        calc_size,
        parsed.capacity()
    );
}

// Test that `Table`'s capacity is being calculated properly
#[test]
fn table_capacity() {
    let mut id_field = Field::new(1, "id".to_string(), FieldType::Int64).unwrap();
    id_field.add_opt(FieldOption::PrimaryKey).unwrap();
    id_field.add_opt(FieldOption::Unique).unwrap();
    id_field.add_opt(FieldOption::NotNull).unwrap();
    id_field.add_opt(FieldOption::AutoIncrement).unwrap();

    let mut name_field = Field::new(2, "name".to_string(), FieldType::Text).unwrap();
    name_field.add_opt(FieldOption::NotNull).unwrap();

    let mut another_field = Field::new(3, "another".to_string(), FieldType::Int32).unwrap();
    another_field.add_opt(FieldOption::Unique).unwrap();

    let mut table = Table::new("test_table".to_string()).unwrap();
    table.add_field(id_field).unwrap();
    table.add_field(name_field).unwrap();
    table.add_field(another_field).unwrap();

    let calc_size = table.parse_size();
    let parsed = table.parse().unwrap();

    assert_eq!(
        parsed.len(),
        parsed.capacity(),
        "len: {}, cap: {}",
        parsed.len(),
        parsed.capacity()
    );

    assert_eq!(
        calc_size,
        parsed.capacity(),
        "calc_size: {}, parsed.capacity(): {}",
        calc_size,
        parsed.capacity()
    );
}

// Test various field errors
#[test]
fn field_error() {
    // Name validation error
    let f = Field::new(1, "err name".to_string(), FieldType::Text);
    assert!(f.is_err());

    let err = f.err().unwrap();
    assert!(
        matches!(
            err,
            FieldError::Validation(ValidationError::UnsupportedChars)
        ),
        "err: {:?}, matching: {:?}",
        err,
        FieldError::Validation(ValidationError::UnsupportedChars)
    );

    // Duplicate option error
    let mut f = Field::new(1, "test_field".to_string(), FieldType::Text).unwrap();
    f.add_opt(FieldOption::Unique).unwrap();

    let res = f.add_opt(FieldOption::Unique);
    assert!(res.is_err());

    let err = res.err().unwrap();
    assert!(
        matches!(err, FieldError::DuplicateOption(FieldOption::Unique)),
        "err: {:?}, matching: {:?}",
        err,
        FieldError::DuplicateOption(FieldOption::Unique)
    );

    // Invalid opt for type error
    let mut f = Field::new(1, "test_field".to_string(), FieldType::Text).unwrap();
    let res = f.add_opt(FieldOption::AutoIncrement);

    assert!(res.is_err());

    let err = res.err().unwrap();
    assert!(
        matches!(
            err,
            FieldError::InvalidTypeOption(FieldType::Text, FieldOption::AutoIncrement)
        ),
        "err: {:?}, matching: {:?}",
        err,
        FieldError::InvalidTypeOption(FieldType::Text, FieldOption::AutoIncrement)
    );
}

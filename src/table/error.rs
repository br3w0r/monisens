use crate::tool::validation::ValidationError;

// TODO: человекочитаемые ошибки

#[derive(Debug)]
pub enum TableError {
    Field((String, FieldError)),
    Validation(ValidationError),
}

#[derive(Debug)]
pub enum FieldError {
    DuplicateOption(super::FieldOption),
    Validation(ValidationError),
}

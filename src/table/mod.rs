mod error;
mod field;
mod index;
mod test;

use crate::tool::validation::validate_chars;

pub use error::*;
pub use field::*;
pub use index::*;

pub struct Table {
    name: String,
    fields: Vec<Field>,
    indices: Vec<Index>,
}

impl Table {
    pub fn new(name: String) -> Result<Self, TableError> {
        if let Err(err) = validate_chars(&name) {
            return Err(TableError::Validation(err));
        }

        Ok(Table {
            name,
            fields: Vec::new(),
            indices: Vec::new(),
        })
    }

    pub fn add_field(&mut self, f: Field) -> Result<(), TableError> {
        // TODO: добавить валидацию дубликатов
        self.fields.push(f);

        Ok(())
    }

    pub fn add_index(&mut self, i: Index) {
        // TODO
    }

    // create table "<table_name>" (
    //     <field>,
    //     <field>,
    //     ...
    // );
    pub fn parse_size(&self) -> usize {
        let mut field_size = 0;
        for (i, field) in self.fields.iter().enumerate() {
            field_size += 2 + field.parse_size();
            if i != self.fields.len() - 1 {
                field_size += 1;
            }
        }

        // TODO: add indices

        // <create_table> + <quotes> + <table_name> + <space> + <bracket> +
        // + <fields> + <new_line> + <bracket> + <semicolumn>
        15 + self.name.len() + 2 + field_size + 3
    }

    pub fn parse(&self) -> Result<String, TableError> {
        let mut s = String::with_capacity(self.parse_size());

        s.push_str("CREATE TABLE \"");
        s.push_str(&self.name);
        s.push_str("\" (");

        for (i, field) in self.fields.iter().enumerate() {
            match field.parse() {
                Ok(field_str) => {
                    s.push_str("\n\t");
                    s.push_str(&field_str)
                }
                Err(err) => return Err(TableError::Field(field.name().to_owned(), err)),
            }

            if i != self.fields.len() - 1 {
                s.push(',');
            }
        }

        s.push_str("\n);");

        // TODO: add indices

        Ok(s)
    }
}

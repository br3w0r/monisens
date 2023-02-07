use crate::tool::validation::validate_chars;
use std::collections::HashSet;

use super::error::FieldError;

pub struct Field {
    name: String,
    typ: FieldType,
    opts: HashSet<FieldOption>,
}

impl Field {
    pub fn new(name: String, typ: FieldType) -> Result<Self, FieldError> {
        if let Err(err) = validate_chars(&name) {
            return Err(FieldError::Validation(err));
        }

        Ok(Field {
            name,
            typ,
            opts: HashSet::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_opt(&mut self, opt: FieldOption) -> Result<(), FieldError> {
        opt.validate_type(&self.typ)?;

        match self.opts.get(&opt) {
            None => {
                self.opts.insert(opt);
                return Ok(());
            }
            Some(o) => Err(FieldError::DuplicateOption(o.clone())),
        }
    }

    pub fn parse_size(&self) -> usize {
        let mut opts_size = 0;
        for opt in self.opts.iter() {
            opts_size += opt.parse_size();
        }

        // <quotes_count> + <name_len> + <space> + <type_len> + <spaces_between_opts> + <opts_len>
        2 + self.name.len() + 1 + self.typ.parse_size() + self.opts.len() + opts_size
    }

    pub fn parse(&self) -> Result<String, FieldError> {
        let mut s = String::with_capacity(self.parse_size());

        s.push('"');
        s.push_str(&self.name);
        s.push_str("\" ");
        s.push_str(&self.typ.parse());
        for opt in self.opts.iter() {
            s.push(' ');
            s.push_str(&opt.parse());
        }

        Ok(s)
    }
}

#[derive(Clone, Debug)]
pub enum FieldType {
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Timestamp,
    Text,
    JSON,
}

impl FieldType {
    pub fn parse_size(&self) -> usize {
        match *self {
            FieldType::Int16 | FieldType::Int32 | FieldType::Int64 => 4,
            FieldType::Float32 | FieldType::Float64 => 6,
            FieldType::Timestamp => 9,
            FieldType::Text => 4,
            FieldType::JSON => 5,
        }
    }

    pub fn parse(&self) -> &'static str {
        match *self {
            FieldType::Int16 => "int2",
            FieldType::Int32 => "int4",
            FieldType::Int64 => "int8",
            FieldType::Float32 => "float4",
            FieldType::Float64 => "float8",
            FieldType::Timestamp => "timestamp",
            FieldType::Text => "text",
            FieldType::JSON => "jsonb",
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum FieldOption {
    PrimaryKey,
    Unique,
    NotNull,
    AutoIncrement,
    // Default(T?), // пока не знаю, как реализовать
}

impl FieldOption {
    pub fn parse_size(&self) -> usize {
        match *self {
            FieldOption::PrimaryKey => 11,
            FieldOption::Unique => 6,
            FieldOption::NotNull => 8,
            FieldOption::AutoIncrement => 32,
        }
    }

    pub fn parse(&self) -> &str {
        match *self {
            FieldOption::PrimaryKey => "PRIMARY KEY",
            FieldOption::Unique => "UNIQUE",
            FieldOption::NotNull => "NOT NULL",
            FieldOption::AutoIncrement => "GENERATED BY DEFAULT AS IDENTITY",
        }
    }

    pub fn validate_type(&self, typ: &FieldType) -> Result<(), FieldError> {
        if *self == FieldOption::AutoIncrement {
            return match typ {
                FieldType::Int32 => Ok(()),
                FieldType::Int64 => Ok(()),
                _ => Err(FieldError::InvalidTypeOption(typ.clone(), self.clone())),
            };
        }

        Ok(())
    }
}

pub mod builder;
pub mod error;
pub mod sqlizer;

use builder::Builder;
use sqlizer::{Part, PredType, Sqlizer};
use sqlx::{Database, Encode, Statement, Type};
use std::any::Any;
use std::error::Error;
use std::rc::Rc;

pub struct StatementBuilder<'a> {
    b: Builder<'a>,
}

impl<'a> StatementBuilder<'a> {
    pub fn new() -> Self {
        Self { b: Builder::new() }
    }

    pub fn table(&mut self, table: String) -> &mut Self {
        self.b.set(
            "table".to_string(),
            Part::new(PredType::String(table), None),
        );

        self
    }

    pub fn columns(&mut self, column: String) -> &mut Self {
        // TODO: multiple columns
        self.b.extend("columns", Part::new(PredType::String(column), None));

        self
    }

    pub fn whereq(&mut self, sq: &'a dyn Sqlizer<'a>) -> &mut Self {
        self.b
            .extend("where", sq)
            .expect("failed to extend 'where' statement");

        self
    }

    pub fn select(self) -> SelectBuilder<'a> {
        self
    }
}

type SelectBuilder<'a> = StatementBuilder<'a>;

impl<'a> SelectBuilder<'a> {
    fn sql_raw(&self) -> Result<(String, Option<Vec<Rc<dyn Any>>>), Box<dyn Error>> {
        let mut sql = String::new();
        let mut args = Vec::new();

        sql.push_str("SELECT ");

        let columns = self.b.get_vec("columns");
        if let Some(columns) = columns {
            if columns.len() > 0 {
                append_sql(&columns, &mut sql, ", ", &mut args)?;
            }
        }

        let from = self.b.get("table");
        if let Some(from) = from {
            sql.push_str(" FROM ");
            append_sql(&vec![from], &mut sql, ", ", &mut args)?;
        }

        Ok((sql, Some(args)))
    }
}

impl<'a> Sqlizer<'a> for SelectBuilder<'a> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<dyn Any>>>), Box<dyn Error>> {
        self.sql_raw()
    }
}

fn append_sql<'a>(
    parts: &Vec<Rc<dyn 'a + Sqlizer<'a>>>,
    s: &mut String,
    sep: &str,
    args: &mut Vec<Rc<dyn Any>>,
) -> Result<(), Box<dyn Error>> {
    for (i, p) in parts.iter().enumerate() {
        let (part_sql, part_args) = p.sql()?;

        if part_sql.len() == 0 {
            continue;
        }

        if i > 0 {
            s.push_str(sep);
        }

        s.push_str(&part_sql);

        if let Some(v) = part_args {
            args.extend(v.iter().map(|x| Rc::clone(x)));
        }
    }

    Ok(())
}

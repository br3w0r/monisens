pub mod builder;
pub mod error;
mod expr;
pub mod integration;
pub mod sqlizer;
mod tool;

use builder::Builder;
pub use expr::*;
use sqlizer::{Part, PredType, Sqlizer};
use sqlx::{Database, Encode, Statement, Type};
use std::any::Any;
use std::error::Error;
use std::rc::Rc;

pub struct StatementBuilder<A> {
    b: Builder<A>,
}

impl<A: 'static> StatementBuilder<A> {
    pub fn new() -> Self {
        Self { b: Builder::new() }
    }

    pub fn table(&mut self, table: String) -> &mut Self {
        self.b.set(
            "table".to_string(),
            Rc::new(Part::<A>::new(PredType::<A>::String(table), None)),
        );

        self
    }

    pub fn column(&mut self, column: String) -> &mut Self {
        self.b
            .push(
                "columns",
                Rc::new(Part::new(PredType::String(column), None)),
            )
            .expect("failed to extend 'columns' statement");

        self
    }

    pub fn columns(&mut self, mut columns: Vec<String>) -> &mut Self {
        let mut v: Vec<Rc<dyn Sqlizer<A>>> = Vec::with_capacity(columns.len());
        for i in columns.drain(0..) {
            self.b
                .push(
                    "columns",
                    Rc::new(Part::<A>::new(PredType::String(i), None)),
                )
                .expect("failed to extend 'columns' statement");
        }

        self
    }

    pub fn whereq(&mut self, sq: Rc<dyn Sqlizer<A>>) -> &mut Self {
        self.b
            .push("where", sq)
            .expect("failed to extend 'where' statement");

        self
    }

    pub fn select(self) -> SelectBuilder<A> {
        self
    }
}

type SelectBuilder<A> = StatementBuilder<A>;

impl<A: 'static> SelectBuilder<A> {
    fn sql_raw(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        let mut sql = String::new();
        let mut args = Vec::new();

        sql.push_str("SELECT ");

        if let Some(columns) = self.b.get_vec("columns") {
            if columns.len() > 0 {
                tool::append_sql(&columns, &mut sql, ", ", &mut args)?;
            }
        }

        if let Some(from) = self.b.get("table") {
            sql.push_str(" FROM ");
            tool::append_sql(&vec![from], &mut sql, ", ", &mut args)?;
        }

        if let Some(wher) = self.b.get_vec("where") {
            if wher.len() > 0 {
                sql.push_str(" WHERE ");
                tool::append_sql(&wher, &mut sql, " AND ", &mut args)?;
            }
        }

        Ok((tool::replace_pos_placeholders(&sql, "$"), Some(args)))
    }
}

impl<A: 'static> Sqlizer<A> for SelectBuilder<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        self.sql_raw()
    }
}

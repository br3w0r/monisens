pub mod builder;
pub mod error;
mod expr;
pub mod integration;
pub mod sqlizer;
mod tool;

use builder::Builder;
pub use expr::*;
use sqlizer::{Part, PredType, Sqlizer, Values};
use std::error::Error;
use std::rc::Rc;

use self::error::{DeleteError, InsertError, SelectError};

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
            Rc::new(Part::new(PredType::String(table), None)),
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

    pub fn columns(&mut self, columns: &[String]) -> &mut Self {
        for i in columns {
            self.column(i.into());
        }

        self
    }

    pub fn whereq(&mut self, sq: Rc<dyn Sqlizer<A>>) -> &mut Self {
        self.b
            .push("where", sq.into())
            .expect("failed to extend 'where' statement");

        self
    }

    pub fn set(&mut self, vals: Vec<A>) -> &mut Self {
        self.b
            .push("values", Rc::new(Values::from(vals)))
            .expect("failed to extend 'values' statement");

        self
    }

    /// Build `SELECT` query
    /// ```no_run
    /// let mut b = sq::StatementBuilder::new();
    /// b.table("test_parse_table".to_string())
    ///     .columns(&["id".into(), "name".into()])
    ///     .whereq(sq::gt("id".to_string(), 2));
    /// 
    /// let b = b.select();
    /// ```
    pub fn select(self) -> SelectBuilder<A> {
        SelectBuilder(self)
    }

    /// Build `INSERT` query
    /// ```no_run
    /// let mut b = sq::StatementBuilder::new();
    /// b.table("test_parse_table".into())
    ///     .column("name".into())
    ///     .set(vec!["foo".into()]).
    ///     .set(vec!["bar".into()]);
    /// 
    /// let b = b.insert();
    /// ```
    pub fn insert(self) -> InsertBuilder<A> {
        InsertBuilder(self)
    }

    pub fn delete(self) -> DeleteBuilder<A> {
        DeleteBuilder(self)
    }
}

pub struct SelectBuilder<A>(StatementBuilder<A>);

impl<A: 'static> Sqlizer<A> for SelectBuilder<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        let columns = match self.0.b.get_vec("columns") {
            Some(cols) => {
                if cols.len() == 0 {
                    return Err(SelectError::NoColumns.into());
                }

                Ok(cols)
            }
            None => Err(SelectError::NoColumns),
        }?;

        let mut sql = String::new();
        let mut args = Vec::new();

        sql.push_str("SELECT ");

        tool::append_sql(&columns, &mut sql, ", ", &mut args)?;

        if let Some(from) = self.0.b.get("table") {
            sql.push_str(" FROM ");
            tool::append_sql(&vec![from], &mut sql, ", ", &mut args)?;
        }

        if let Some(wher) = self.0.b.get_vec("where") {
            if wher.len() > 0 {
                sql.push_str(" WHERE ");
                tool::append_sql(&wher, &mut sql, " AND ", &mut args)?;
            }
        }

        Ok((tool::replace_pos_placeholders(&sql, "$"), Some(args)))
    }
}

pub struct InsertBuilder<A>(StatementBuilder<A>);

impl<A: 'static> Sqlizer<A> for InsertBuilder<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        let into = match self.0.b.get("table") {
            Some(v) => Ok(v),
            None => Err(InsertError::NoTable),
        }?;

        let values = match self.0.b.get_vec("values") {
            Some(v) => {
                if v.len() == 0 {
                    return Err(InsertError::NoValues.into());
                }

                Ok(v)
            }
            None => Err(InsertError::NoValues),
        }?;

        let mut sql = String::new();
        let mut args = Vec::new();

        sql.push_str("INSERT INTO ");

        {
            let (s, _) = into.sql()?;
            sql.push_str(&s);
        }

        if let Some(columns) = self.0.b.get_vec("columns") {
            if columns.len() > 0 {
                sql.push('(');
                if let Err(err) = tool::append_sql(&columns, &mut sql, ", ", &mut args) {
                    return Err(err);
                }
                sql.push(')');
            }
        }

        sql.push_str(" VALUES ");
        tool::append_sql(&values, &mut sql, ", ", &mut args)?;

        Ok((tool::replace_pos_placeholders(&sql, "$"), Some(args)))
    }
}

pub struct DeleteBuilder<A>(StatementBuilder<A>);

impl<A: 'static> Sqlizer<A> for DeleteBuilder<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        let into = match self.0.b.get("table") {
            Some(v) => Ok(v),
            None => Err(DeleteError::NoTable),
        }?;

        let mut sql = String::new();
        let mut args = Vec::new();

        sql.push_str("DELETE FROM ");

        {
            let (s, _) = into.sql()?;
            sql.push_str(&s);
        }

        if let Some(wher) = self.0.b.get_vec("where") {
            if wher.len() > 0 {
                sql.push_str(" WHERE ");
                tool::append_sql(&wher, &mut sql, " AND ", &mut args)?;
            }
        }

        Ok((tool::replace_pos_placeholders(&sql, "$"), Some(args)))
    }
}

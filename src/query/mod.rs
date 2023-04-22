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

use self::error::{DeleteError, InsertError, SelectError, UpdateError};

const TABLE: &str = "table";
const COLUMNS: &str = "columns";
const WHERE: &str = "where";
const VALUES: &str = "values";
const SET: &str = "set";
const ORDER: &str = "order";
const LIMIT: &str = "limit";

pub struct StatementBuilder<A> {
    b: Builder<A>,
}

impl<A: 'static> StatementBuilder<A> {
    pub fn new() -> Self {
        Self { b: Builder::new() }
    }

    pub fn table(&mut self, table: String) -> &mut Self {
        self.b.set(
            TABLE.to_string(),
            Rc::new(Part::new(PredType::String(table), None)),
        );

        self
    }

    pub fn column<S: Into<String>>(&mut self, column: S) -> &mut Self {
        self.b
            .push(
                COLUMNS,
                Rc::new(Part::new(PredType::String(column.into()), None)),
            )
            .expect("failed to extend 'columns' statement");

        self
    }

    pub fn columns<S: AsRef<str>>(&mut self, columns: &[S]) -> &mut Self {
        for i in columns.into_iter() {
            self.column(i.as_ref());
        }

        self
    }

    pub fn whereq(&mut self, sq: Rc<dyn Sqlizer<A>>) -> &mut Self {
        self.b
            .push(WHERE, sq.into())
            .expect("failed to extend 'where' statement");

        self
    }

    // `values` appends 'VALUES' clause for insert statement
    pub fn values(&mut self, vals: Vec<A>) -> &mut Self {
        self.b
            .push(VALUES, Rc::new(Values::from(vals)))
            .expect("failed to extend 'values' statement");

        self
    }

    // `set` appends 'SET' clause for update statement
    pub fn set(&mut self, column: String, value: A) -> &mut Self {
        self.b
            .push(SET, SetExpr::new(column, value))
            .expect("failed to extend 'set' statement");

        self
    }

    pub fn order(&mut self, order: String) -> &mut Self {
        self.b.set(
            ORDER.to_string(),
            Rc::new(Part::new(PredType::String(order), None)),
        );

        self
    }

    pub fn limit(&mut self, limit: i32) -> &mut Self {
        self.b.set(
            LIMIT.to_string(),
            Rc::new(Part::new(
                PredType::String("LIMIT ".to_string() + &limit.to_string()),
                None,
            )),
        );

        self
    }

    /// Build `SELECT` query
    /// ```no_run
    /// use query::integration::isqlx as sq;
    ///
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
    /// use query::integration::isqlx as sq;
    ///
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

    pub fn update(self) -> UpdateBuilder<A> {
        UpdateBuilder(self)
    }
}

pub struct SelectBuilder<A>(StatementBuilder<A>);

impl<A: 'static> Sqlizer<A> for SelectBuilder<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        let columns = match self.0.b.get_vec(COLUMNS) {
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

        if let Some(from) = self.0.b.get(TABLE) {
            sql.push_str(" FROM ");
            tool::append_sql(&vec![from], &mut sql, ", ", &mut args)?;
        }

        if let Some(wher) = self.0.b.get_vec(WHERE) {
            if wher.len() > 0 {
                sql.push_str(" WHERE ");
                tool::append_sql(&wher, &mut sql, " AND ", &mut args)?;
            }
        }

        if let Some(order) = self.0.b.get(ORDER) {
            sql.push_str(" ORDER BY ");
            sql.push_str(&order.sql()?.0);
        }

        if let Some(limit) = self.0.b.get(LIMIT) {
            sql.push(' ');
            sql.push_str(&limit.sql()?.0);
        }

        Ok((tool::replace_pos_placeholders(&sql, "$"), Some(args)))
    }
}

pub struct InsertBuilder<A>(StatementBuilder<A>);

impl<A: 'static> Sqlizer<A> for InsertBuilder<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        let into = self.0.b.get(TABLE).ok_or(InsertError::NoTable)?;

        let values = match self.0.b.get_vec(VALUES) {
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

        if let Some(columns) = self.0.b.get_vec(COLUMNS) {
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
        let into = self.0.b.get(TABLE).ok_or(DeleteError::NoTable)?;

        let mut sql = String::new();
        let mut args = Vec::new();

        sql.push_str("DELETE FROM ");

        {
            let (s, _) = into.sql()?;
            sql.push_str(&s);
        }

        if let Some(wher) = self.0.b.get_vec(WHERE) {
            if wher.len() > 0 {
                sql.push_str(" WHERE ");
                tool::append_sql(&wher, &mut sql, " AND ", &mut args)?;
            }
        }

        Ok((tool::replace_pos_placeholders(&sql, "$"), Some(args)))
    }
}

pub struct UpdateBuilder<A>(StatementBuilder<A>);

impl<A: 'static> Sqlizer<A> for UpdateBuilder<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        let into = self.0.b.get(TABLE).ok_or(UpdateError::NoTable)?;
        let sets = match self.0.b.get_vec(SET) {
            Some(v) => {
                if v.len() == 0 {
                    return Err(UpdateError::NoSets.into());
                }

                Ok(v)
            }
            None => Err(UpdateError::NoSets),
        }?;

        let mut sql = String::new();
        let mut args = Vec::new();

        sql.push_str("UPDATE ");
        {
            let (s, _) = into.sql()?;
            sql.push_str(&s);
        }

        sql.push_str(" SET ");
        tool::append_sql(&sets, &mut sql, ", ", &mut args)?;

        if let Some(wher) = self.0.b.get_vec(WHERE) {
            if wher.len() > 0 {
                sql.push_str(" WHERE ");
                tool::append_sql(&wher, &mut sql, " AND ", &mut args)?;
            }
        }

        Ok((tool::replace_pos_placeholders(&sql, "$"), Some(args)))
    }
}

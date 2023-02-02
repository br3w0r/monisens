use std::error::Error;

use sqlx::{
    postgres::{PgPoolOptions, PgQueryResult, PgRow},
    Executor, FromRow, Pool, Postgres,
};

use crate::query::integration::isqlx::{self as sq, ArgType};
use crate::{query::sqlizer::Sqlizer, table::Table};

pub struct Repository {
    pool: Pool<Postgres>,
}

impl Repository {
    pub async fn new(dsn: &str) -> Result<Self, Box<dyn Error>> {
        // TODO: advanced configuration?
        let pool = PgPoolOptions::new().max_connections(5).connect(dsn).await?;

        Ok(Self { pool })
    }

    pub async fn create_table(&self, table: Table) -> Result<(), Box<dyn Error>> {
        let table_q = table.parse()?;

        sqlx::query(&table_q).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn exec<S: Sqlizer<Box<dyn ArgType>>>(
        &self,
        q: S,
    ) -> Result<PgQueryResult, Box<dyn Error>> {
        let (sql, args) = q.sql()?;
        let res = sq::query(&sql, &args).execute(&self.pool).await?;

        Ok(res)
    }

    pub async fn get<S, T>(&self, q: S) -> Result<T, Box<dyn Error>>
    where
        S: Sqlizer<Box<dyn ArgType>>,
        T: for<'r> FromRow<'r, PgRow>,
    {
        let (sql, args) = q.sql()?;
        let row = sq::query(&sql, &args).fetch_one(&self.pool).await?;

        let res = T::from_row(&row)?;

        Ok(res)
    }

    pub async fn select<S, T>(&self, q: S) -> Result<Vec<T>, Box<dyn Error>>
    where
        S: Sqlizer<Box<dyn ArgType>>,
        T: for<'r> FromRow<'r, PgRow>,
    {
        let (sql, args) = q.sql()?;
        let rows = sq::query(&sql, &args).fetch_all(&self.pool).await?;

        let mut res = Vec::with_capacity(rows.len());

        for row in rows {
            let entry = T::from_row(&row)?;
            res.push(entry);
        }

        Ok(res)
    }

    pub async fn exec_raw(&self, sql: &str) -> Result<PgQueryResult, Box<dyn Error>> {
        let res = sqlx::query(sql).execute(&self.pool).await?;

        Ok(res)
    }

    pub async fn migrate(&self) -> Result<(), Box<dyn Error>> {
        sqlx::migrate!().run(&self.pool).await?;

        Ok(())
    }
}

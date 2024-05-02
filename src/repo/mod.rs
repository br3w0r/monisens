mod error;
mod querier;

use std::time::Duration;

use sqlx::{
    postgres::{PgPoolOptions, PgQueryResult, PgRow},
    FromRow, Pool, Postgres,
};

use crate::query::integration::isqlx::ArgType;
use crate::{query::sqlizer::Sqlizer, table::Table};

pub use error::*;

#[derive(Clone)]
pub struct Repository {
    pool: Pool<Postgres>,
}

impl Repository {
    pub async fn new(dsn: &str) -> Result<Self, RepoError> {
        // TODO: advanced configuration?
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(dsn)
            .await?;

        Ok(Self { pool })
    }

    pub async fn create_table(&self, table: Table) -> Result<(), RepoError> {
        querier::create_table(&self.pool, table).await
    }

    pub async fn exec<S: Sqlizer<Box<dyn ArgType>>>(
        &self,
        q: S,
    ) -> Result<PgQueryResult, RepoError> {
        querier::exec(&self.pool, q).await
    }

    pub async fn get<S, T>(&self, q: S) -> Result<T, RepoError>
    where
        S: Sqlizer<Box<dyn ArgType>>,
        T: for<'r> FromRow<'r, PgRow>,
    {
        querier::get(&self.pool, q).await
    }

    pub async fn select<S, T>(&self, q: S) -> Result<Vec<T>, RepoError>
    where
        S: Sqlizer<Box<dyn ArgType>>,
        T: for<'r> FromRow<'r, PgRow>,
    {
        querier::select(&self.pool, q).await
    }

    pub async fn exec_raw(&self, sql: &str) -> Result<PgQueryResult, RepoError> {
        querier::exec_raw(&self.pool, sql).await
    }

    pub async fn migrate(&self) -> Result<(), RepoError> {
        sqlx::migrate!().run(&self.pool).await?;

        Ok(())
    }

    pub async fn tx(&self) -> Result<Transaction, RepoError> {
        let tx = self.pool.begin().await?;

        Ok(Transaction(tx))
    }
}

pub struct Transaction<'c>(sqlx::Transaction<'c, Postgres>);

impl<'c> Transaction<'c> {
    pub async fn create_table(&mut self, table: Table) -> Result<(), RepoError> {
        querier::create_table(&mut self.0, table).await
    }

    pub async fn exec<S: Sqlizer<Box<dyn ArgType>>>(
        &mut self,
        q: S,
    ) -> Result<PgQueryResult, RepoError> {
        querier::exec(&mut self.0, q).await
    }

    pub async fn get<S, T>(&mut self, q: S) -> Result<T, RepoError>
    where
        S: Sqlizer<Box<dyn ArgType>>,
        T: for<'r> FromRow<'r, PgRow>,
    {
        querier::get(&mut self.0, q).await
    }

    pub async fn select<S, T>(&mut self, q: S) -> Result<Vec<T>, RepoError>
    where
        S: Sqlizer<Box<dyn ArgType>>,
        T: for<'r> FromRow<'r, PgRow>,
    {
        querier::select(&mut self.0, q).await
    }

    pub async fn exec_raw(&mut self, sql: &str) -> Result<PgQueryResult, RepoError> {
        querier::exec_raw(&mut self.0, sql).await
    }

    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.0.commit().await
    }

    pub async fn rollback(self) -> Result<(), sqlx::Error> {
        self.0.rollback().await
    }
}

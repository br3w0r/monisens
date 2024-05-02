use sqlx::{
    postgres::{PgQueryResult, PgRow},
    Executor, FromRow, Postgres,
};

use crate::{
    query::{
        integration::isqlx::{self as sq, ArgType},
        sqlizer::Sqlizer,
    },
    table::Table,
};

use super::error::RepoError;

pub async fn create_table<'e, E>(e: E, table: Table) -> Result<(), RepoError>
where
    E: Executor<'e, Database = Postgres>,
{
    let table_q = table.parse()?;

    sqlx::query(&table_q).execute(e).await?;

    Ok(())
}

pub async fn exec<'e, E, S: Sqlizer<Box<dyn ArgType>>>(
    e: E,
    q: S,
) -> Result<PgQueryResult, RepoError>
where
    E: Executor<'e, Database = Postgres>,
{
    let (sql, args) = q.sql()?;
    let res = sq::query(&sql, &args).execute(e).await?;

    Ok(res)
}

pub async fn get<'e, E, S, T>(e: E, q: S) -> Result<T, RepoError>
where
    E: Executor<'e, Database = Postgres>,
    S: Sqlizer<Box<dyn ArgType>>,
    T: for<'r> FromRow<'r, PgRow>,
{
    let (sql, args) = q.sql()?;
    let row = sq::query(&sql, &args).fetch_one(e).await?;

    let res = T::from_row(&row)?;

    Ok(res)
}

pub async fn select<'e, E, S, T>(e: E, q: S) -> Result<Vec<T>, RepoError>
where
    E: Executor<'e, Database = Postgres>,
    S: Sqlizer<Box<dyn ArgType>>,
    T: for<'r> FromRow<'r, PgRow>,
{
    let (sql, args) = q.sql()?;
    let rows = sq::query(&sql, &args).fetch_all(e).await?;

    let mut res = Vec::with_capacity(rows.len());

    for row in rows {
        let entry = T::from_row(&row)?;
        res.push(entry);
    }

    Ok(res)
}

pub async fn exec_raw<'e, E>(e: E, sql: &str) -> Result<PgQueryResult, RepoError>
where
    E: Executor<'e, Database = Postgres>,
{
    let res = sqlx::query(sql).execute(e).await?;

    Ok(res)
}

use sqlx::error::Error as SqlxError;
use thiserror::Error;

use crate::controller::error::{CommonError, ErrorType};

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("failed to create table: {0}")]
    Table(crate::table::TableError),
    #[error("failed to migrate: {0}")]
    Migrate(sqlx::migrate::MigrateError),
    #[error("failed to execute query: {0}")]
    Query(sqlx::error::Error),
    #[error("other error: {0}")]
    Other(#[source] Box<dyn std::error::Error>),
}

impl RepoError {
    pub fn to_common_err<S>(self, msg: S) -> CommonError
    where
        S: Into<String>,
    {
        CommonError::new(self.get_ctrl_type(), msg).with_source(Box::new(self))
    }

    pub fn get_ctrl_type(&self) -> ErrorType {
        match &self {
            RepoError::Query(err) => match err {
                SqlxError::RowNotFound => ErrorType::NotFound,
                SqlxError::PoolTimedOut => ErrorType::Timeout,
                SqlxError::Io(_) => ErrorType::IO,
                SqlxError::Tls(_) => ErrorType::IO,
                SqlxError::Protocol(_) => ErrorType::IO,
                SqlxError::Database(db_err) => {
                    return match db_err.code() {
                        Some(code) => {
                            let val = code.as_ref();
                            match val {
                                "23505" => ErrorType::AlreadyExists,
                                _ => ErrorType::Internal,
                            }
                        }
                        None => ErrorType::Internal,
                    };
                }
                _ => ErrorType::Internal,
            },
            _ => ErrorType::Internal,
        }
    }
}

impl From<crate::table::TableError> for RepoError {
    fn from(e: crate::table::TableError) -> Self {
        Self::Table(e)
    }
}

impl From<sqlx::migrate::MigrateError> for RepoError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        Self::Migrate(e)
    }
}

impl From<sqlx::error::Error> for RepoError {
    fn from(e: sqlx::error::Error) -> Self {
        Self::Query(e)
    }
}

impl From<Box<dyn std::error::Error>> for RepoError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        Self::Other(e)
    }
}

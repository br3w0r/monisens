use core::fmt;
use std::{backtrace::Backtrace, error::Error};

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Unknown,
    Internal,
    InvalidInput,
    NotFound,
    AlreadyExists,
    FailedPrecondition,
    Timeout,
    /// E.g. connection lost, disk corruption, etc.
    IO,
}

#[derive(Debug)]
pub struct CommonError {
    pub error_type: ErrorType,
    pub msg: String,
    pub backtrace: Backtrace,
    pub source: Option<Box<dyn std::error::Error>>,
}

impl CommonError {
    pub fn new<S>(error_type: ErrorType, message: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            error_type,
            msg: message.into(),
            backtrace: Backtrace::capture(),
            source: None,
        }
    }

    pub fn with_source<E: Into<Box<dyn std::error::Error>>>(mut self, source: E) -> Self {
        self.source = Some(source.into());
        self
    }
}

impl fmt::Display for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CommonError: type: '{:?}', message: '{}'",
            self.error_type, self.msg
        )?;

        write!(f, "\nSource: ")?;
        match self.source {
            Some(ref e) => write!(f, "{}", e)?,
            None => write!(f, "None")?,
        }

        write!(
            f,
            "\nBacktrace:\n\
            {}",
            self.backtrace
        )
    }
}

impl Error for CommonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

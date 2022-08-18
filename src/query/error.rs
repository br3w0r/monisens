use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum BuilderError {
    NotVec,
    NotAny,
}

pub struct PartErr {
    msg: &'static str,
}

impl PartErr {
    pub fn new(msg: &'static str) -> Self {
        Self { msg }
    }
}

impl fmt::Display for PartErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl fmt::Debug for PartErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl Error for PartErr {}

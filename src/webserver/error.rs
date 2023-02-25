use crate::debug_from_display;
use thiserror::Error;

use std::fmt;

#[derive(Error)]
pub enum ConfigError {
    #[error("static_dir is empty")]
    EmptyStaticDir,
}

debug_from_display!(ConfigError);

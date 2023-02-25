use super::error::ConfigError;

struct Config {
    static_dir: String,
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.static_dir.len() == 0 {
            return Err(ConfigError::EmptyStaticDir);
        }

        Ok(())
    }
}

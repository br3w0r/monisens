use std::path::PathBuf;

#[derive(Clone)]
pub struct AppConfig {
    static_dir: PathBuf,
    index_file: PathBuf,
    favicon_file: PathBuf,
}

impl AppConfig {
    pub fn new(app_data_dir: PathBuf) -> AppConfig {
        let static_dir = app_data_dir.join("static");
        let index_file = app_data_dir.join("index.html");
        let favicon_file = app_data_dir.join("favicon.ico");

        AppConfig {
            static_dir,
            index_file,
            favicon_file,
        }
    }

    pub fn static_dir(&self) -> &PathBuf {
        &self.static_dir
    }

    pub fn index_file(&self) -> &PathBuf {
        &self.index_file
    }

    pub fn favicon_file(&self) -> &PathBuf {
        &self.favicon_file
    }
}

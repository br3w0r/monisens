pub struct Conf {
    repo_dsn: String,
}

impl Conf {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_repo_dsn(mut self, repo_dsn: String) -> Self {
        self.repo_dsn = repo_dsn;

        self
    }

    pub fn get_repo_dsn(&self) -> &String {
        &self.repo_dsn
    }
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            repo_dsn: Default::default(),
        }
    }
}

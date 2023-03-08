use actix_files::NamedFile;
use actix_web::{get, HttpRequest, Result};
use std::path::PathBuf;

// TODO: move to a dynamic config
const STATIC_DIR: &str = "./app_data/static/";
const INDEX_FILE: &str = "./app_data/index.html";
const FAVICON_FILE: &str = "./app_data/favicon.ico";

#[get("/app/{_:.*}")]
pub async fn index() -> Result<NamedFile> {
    serve_file(INDEX_FILE)
}

#[get("/static/{filename:.*}")]
pub async fn serve_static(req: HttpRequest) -> Result<NamedFile> {
    serve_file(&format!(
        "{}{}",
        STATIC_DIR,
        req.match_info().query("filename")
    ))
}

#[get("/favicon.ico")]
pub async fn serve_favicon() -> Result<NamedFile> {
    serve_file(FAVICON_FILE)
}

fn serve_file(path: &str) -> Result<NamedFile> {
    let path: PathBuf = path.parse().unwrap();

    Ok(NamedFile::open(path)?)
}

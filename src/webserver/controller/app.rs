use crate::webserver::model::AppState;
use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, Result};

#[get("/app/{_:.*}")]
pub async fn index(data: web::Data<AppState>) -> Result<NamedFile> {
    serve_file(data.conf.index_file())
}

#[get("/static/{filename:.*}")]
pub async fn serve_static(data: web::Data<AppState>, req: HttpRequest) -> Result<NamedFile> {
    let filename = req.match_info().query("filename");
    let path = data.conf.static_dir().join(filename);

    serve_file(path)
}

#[get("/favicon.ico")]
pub async fn serve_favicon(data: web::Data<AppState>) -> Result<NamedFile> {
    serve_file(data.conf.favicon_file())
}

fn serve_file<P: AsRef<std::path::Path>>(path: P) -> Result<NamedFile> {
    Ok(NamedFile::open(path)?)
}

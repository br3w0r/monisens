use actix_web::{get, HttpRequest};

#[get("/{_:.*}")]
pub async fn index() -> String {
    // TODO: serve static
    todo!()
}

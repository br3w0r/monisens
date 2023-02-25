pub mod app {
    use actix_web::{get, HttpRequest};

    #[get("/{any:.*}")]
    pub async fn index(req: HttpRequest) -> String {
        // TODO: serve static
        let arg = req.match_info().query("any").to_string();

        arg
    }
}

pub mod service {
    use actix_web::{get, web};

    use crate::webserver::model::State;

    #[get("/")]
    pub async fn index(data: web::Data<State>) -> String {
        "Hello, world!".into()
    }
}

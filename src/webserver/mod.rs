mod config;
mod error;
mod model;
mod route;

use std::error::Error;

use actix_web::{web, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::controller;

use model::*;
use route::*;

pub async fn start_server(ctrl: controller::Controller) -> Result<(), Box<dyn Error>> {
    #[derive(OpenApi)]
    #[openapi(
        paths(service::index, service::test_save_files),
        components(schemas(model::contract::TestUploadForm))
    )]
    struct ApiDoc;

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/service")
                    .app_data(web::Data::new(State { ctrl: ctrl.clone() }))
                    .service(web::redirect("", "/service/"))
                    .service(service::index)
                    .service(service::test_save_files),
            )
            .service(
                web::scope("/app")
                    .service(app::index)
                    .service(web::redirect("", "/app/")),
            )
            .service(SwaggerUi::new("/docs/{_:.*}").url("/swagger.json", ApiDoc::openapi()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}

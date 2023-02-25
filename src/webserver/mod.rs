mod config;
mod error;
mod model;
mod route;

use std::error::Error;

use actix_web::{web, App, HttpServer};

use crate::controller;

use model::*;
use route::*;

pub async fn start_server(ctrl: controller::Controller) -> Result<(), Box<dyn Error>> {
    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/service")
                    .app_data(web::Data::new(State { ctrl: ctrl.clone() }))
                    .service(service::index),
            )
            .service(web::scope("/app").service(app::index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}

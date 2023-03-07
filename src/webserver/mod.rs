mod config;
mod controller;
mod error;
mod model;

use std::error::Error;

use actix_web::{web, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use controller::*;
use model::*;

pub async fn start_server(ctrl: crate::controller::Controller) -> Result<(), Box<dyn Error>> {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            service::index,
            service::test_save_files,
            service::start_device_init,
            service::connect_device,
            service::obtain_device_conf_info,
            service::configure_device,
            service::interrupt_device_init,
        ),
        components(schemas(
            contract::TestUploadForm,
            contract::DeviceStartInitRequest,
            contract::DeviceStartInitResponse,
            contract::ConnParamConf,
            contract::ConnParamType,
            contract::ConnectDeviceRequest,
            contract::ConnParam,
            contract::ObtainDeviceConfInfoRequest,
            contract::ObtainDeviceConfInfoResponse,
            contract::DeviceConfInfoEntry,
            contract::DeviceConfInfoEntryType,
            contract::DeviceConfInfoEntryString,
            contract::DeviceConfInfoEntryInt,
            contract::DeviceConfInfoEntryIntRange,
            contract::DeviceConfInfoEntryFloat,
            contract::DeviceConfInfoEntryFloatRange,
            contract::DeviceConfInfoEntryJSON,
            contract::DeviceConfInfoEntryChoiceList,
            contract::ConfigureDeviceRequest,
            contract::DeviceConfEntry,
            contract::DeviceConfType,
            contract::InterruptDeviceInitRequest,
        ))
    )]
    struct ApiDoc;

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/service")
                    .app_data(web::Data::new(State { ctrl: ctrl.clone() }))
                    .service(web::redirect("", "/service/"))
                    .service(service::index)
                    .service(service::test_save_files)
                    .service(service::start_device_init)
                    .service(service::connect_device)
                    .service(service::obtain_device_conf_info)
                    .service(service::configure_device)
                    .service(service::interrupt_device_init),
            )
            .service(
                web::scope("/app")
                    .service(app::index)
                    .service(web::redirect("", "/app/")),
            )
            .service(SwaggerUi::new("/docs/{_:.*}").url("/swagger.json", ApiDoc::openapi()))
            .service(web::redirect("/docs", "/docs/"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}

mod controller;
mod model;

use std::error::Error;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use controller::*;
use model::*;

pub async fn start_server(ctrl: crate::controller::Controller) -> Result<(), Box<dyn Error>> {
    #[derive(OpenApi)]
    #[openapi(
        paths(
            service::start_device_init,
            service::connect_device,
            service::obtain_device_conf_info,
            service::configure_device,
            service::interrupt_device_init,
            service::get_sensor_data,
            service::get_device_list,
        ),
        components(schemas(
            contract::TestUploadForm,
            contract::DeviceStartInitRequest,
            contract::DeviceStartInitResponse,
            contract::ConnParamConf,
            contract::ConnParamType,
            contract::ConnectDeviceRequest,
            contract::ConnParam,
            contract::ConnParamValType,
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
            contract::GetSensorDataRequest,
            contract::GetSensorDataResponse,
            contract::Sort,
            contract::SortOrder,
            contract::SensorData,
            contract::GetDeviceListResponse,
            contract::DeviceEntry,
        ))
    )]
    struct ApiDoc;

    HttpServer::new(move || {
        // TODO: replace with conditional code for testing
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .service(
                web::scope("/service")
                    .app_data(web::Data::new(State { ctrl: ctrl.clone() }))
                    .service(service::start_device_init)
                    .service(service::connect_device)
                    .service(service::obtain_device_conf_info)
                    .service(service::configure_device)
                    .service(service::interrupt_device_init)
                    .service(service::get_sensor_data)
                    .service(service::get_device_list),
            )
            .service(app::index)
            .service(web::redirect("/app", "/app/"))
            .service(app::serve_static)
            .service(SwaggerUi::new("/docs/{_:.*}").url("/swagger.json", ApiDoc::openapi()))
            .service(web::redirect("/docs", "/docs/"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}

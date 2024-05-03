pub mod config;

mod controller;
mod model;

use std::error::Error;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use controller::*;
use model::*;

use crate::controller::Controller;
use crate::module::Module;
use crate::service::Service;

pub async fn start_server(
    host: String,
    ctrl: Controller<Service, Module, Module>,
    app_config: config::AppConfig,
) -> Result<(), Box<dyn Error>> {
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
            service::get_device_sensor_info,
            service::save_monitor_conf,
            service::get_monitor_conf_list,
        ),
        components(schemas(
            error::WebError,
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
            contract::SensorData,
            contract::GetDeviceListResponse,
            contract::DeviceEntry,
            contract::GetDeviceSensorInfoRequest,
            contract::GetDeviceSensorInfoResponse,
            contract::SensorInfo,
            contract::SensorDataInfo,
            contract::SensorDataType,
            contract::SaveMonitorConfRequest,
            contract::SaveMonitorConfResponse,
            contract::MonitorType,
            contract::MonitorTypeConf,
            contract::MonitorLogConf,
            contract::SortDir,
            contract::MonitorConfListRequest,
            contract::MonitorConfListFilter,
            contract::MonitorConfListResponse,
            contract::MonitorConfListEntry,
            contract::ConnParamEntryInfo,
            contract::ConnParamChoiceListInfo,
            contract::MonitorLineConf,
        ))
    )]
    struct ApiDoc;

    HttpServer::new(move || {
        // TODO: replace with conditional code for testing
        let cors = Cors::permissive();

        let json_cfg = web::JsonConfig::default()
            .error_handler(|err, _| model::error::WebError::from(err).into());

        App::new()
            .app_data(json_cfg)
            .wrap(cors)
            .service(
                web::scope("/service")
                    .app_data(web::Data::new(ServiceState { ctrl: ctrl.clone() }))
                    .service(service::start_device_init)
                    .service(service::connect_device)
                    .service(service::obtain_device_conf_info)
                    .service(service::configure_device)
                    .service(service::interrupt_device_init)
                    .service(service::get_sensor_data)
                    .service(service::get_device_list)
                    .service(service::get_device_sensor_info)
                    .service(service::get_monitor_conf_list)
                    .service(service::save_monitor_conf),
            )
            .app_data(web::Data::new(AppState {
                conf: app_config.clone(),
            }))
            .service(app::index)
            .service(web::redirect("/app", "/app/"))
            .service(app::serve_static)
            .service(SwaggerUi::new("/docs/{_:.*}").url("/swagger.json", ApiDoc::openapi()))
            .service(web::redirect("/docs", "/docs/"))
            .service(web::redirect("/", "/app/"))
    })
    .bind(host)
    .unwrap()
    .run()
    .await
    .unwrap();

    Ok(())
}

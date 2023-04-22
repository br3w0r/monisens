use actix_multipart::form::MultipartForm;
use actix_web::{post, web, HttpResponse, Responder, Result};
use actix_web_validator::Json;

use crate::webserver::model::{contract, State};

#[utoipa::path(
    context_path = "/service",
    request_body(content = DeviceStartInitRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Ok response with device id and connection params", body = DeviceStartInitResponse),
        (status = 500, description = "Server error response"),
    ),
)]
#[post("/start-device-init")]
pub async fn start_device_init(
    data: web::Data<State>,
    MultipartForm(form): MultipartForm<contract::DeviceStartInitRequest>,
) -> Result<impl Responder> {
    let mut file = tokio::fs::File::open(form.module_file.file.path()).await?;

    let res = data
        .ctrl
        .start_device_init(form.device_name.to_string(), &mut file)
        .await?;

    Ok(web::Json(contract::DeviceStartInitResponse::from(res)))
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = ConnectDeviceRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response"),
        (status = 500, description = "Server error response"),
    ),
)]
#[post("/connect-device")]
pub async fn connect_device(
    data: web::Data<State>,
    mut req: web::Json<contract::ConnectDeviceRequest>,
) -> Result<impl Responder> {
    data.ctrl.connect_device(
        req.device_id,
        req.connect_conf.drain(..).map(|v| v.into()).collect(),
    )?;

    Ok(HttpResponse::Ok())
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = ObtainDeviceConfInfoRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response with device conf info", body = ObtainDeviceConfInfoResponse),
        (status = 500, description = "Server error response"),
    ),
)]
#[post("/obtain-device-conf-info")]
pub async fn obtain_device_conf_info(
    data: web::Data<State>,
    req: web::Json<contract::ObtainDeviceConfInfoRequest>,
) -> Result<impl Responder> {
    let mut res = data.ctrl.obtain_device_conf_info(req.device_id)?;

    Ok(web::Json(contract::ObtainDeviceConfInfoResponse {
        device_conf_info: res.drain(..).map(|v| v.into()).collect(),
    }))
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = ConfigureDeviceRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response"),
        (status = 500, description = "Server error response"),
    ),
)]
#[post("/configure-device")]
pub async fn configure_device(
    data: web::Data<State>,
    mut req: web::Json<contract::ConfigureDeviceRequest>,
) -> Result<impl Responder> {
    data.ctrl
        .configure_device(
            req.device_id,
            req.confs.drain(..).map(|v| v.into()).collect(),
        )
        .await?;

    Ok(HttpResponse::Ok())
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = InterruptDeviceInitRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response"),
        (status = 500, description = "Server error response"),
    ),
)]
#[post("/interrupt-device-init")]
pub async fn interrupt_device_init(
    data: web::Data<State>,
    req: web::Json<contract::InterruptDeviceInitRequest>,
) -> Result<impl Responder> {
    data.ctrl.interrupt_device_init(req.device_id).await?;

    Ok(HttpResponse::Ok())
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = GetSensorDataRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response", body = GetSensorDataResponse),
        (status = 500, description = "Server error response"),
    ),
)]
#[post("/get-sensor-data")]
pub async fn get_sensor_data(
    data: web::Data<State>,
    req: Json<contract::GetSensorDataRequest>,
) -> Result<impl Responder> {
    // TODO: validation: if `limit` is null, `from` mustn't be null. If `from` is not null, `limit` must be null
    let res = data.ctrl.get_sensor_data(req.0.clone().into()).await?;

    Ok(web::Json::<contract::GetSensorDataResponse>(res.into()))
}

#[utoipa::path(
    context_path = "/service",
    responses(
        (status = 200, description = "Ok response", body = GetDeviceListResponse),
        (status = 500, description = "Server error response"),
    ),
)]
#[post("/get-device-list")]
pub async fn get_device_list(data: web::Data<State>) -> Result<impl Responder> {
    let mut res = data.ctrl.get_device_list();

    res.sort_unstable_by(|a, b| a.id.partial_cmp(&b.id).unwrap());

    Ok(web::Json::<contract::GetDeviceListResponse>(res.into()))
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = GetDeviceSensorInfoRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ok response with device conf info", body = GetDeviceSensorInfoResponse),
        (status = 500, description = "Server error response"),
    ),
)]
#[post("/get-device-sensor-info")]
pub async fn get_device_sensor_info(
    data: web::Data<State>,
    req: Json<contract::GetDeviceSensorInfoRequest>,
) -> Result<impl Responder> {
    let res = data.ctrl.get_device_sensor_info(req.device_id)?;

    Ok(web::Json::<contract::GetDeviceSensorInfoResponse>(
        res.into(),
    ))
}

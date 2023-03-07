use std::error::Error;

use actix_multipart::form::MultipartForm;
use actix_web::{get, post, web, HttpResponse, Responder, Result};
use tokio::{fs, io::AsyncWriteExt};

use crate::webserver::model::{contract, State};

#[utoipa::path(
        context_path = "/service",
        responses(
            (status = 200, description = "Hello, world!", body = String)
        )
    )]
#[get("/")]
pub async fn index() -> String {
    "Hello, world!".into()
}

#[utoipa::path(
        context_path = "/service",
        responses(
            (status = 200, description = "Ok response")
        ),
        request_body(content = TestUploadForm, content_type = "multipart/form-data")
    )]
#[post("/test-save-files")]
pub async fn test_save_files(
    MultipartForm(form): MultipartForm<contract::TestUploadForm>,
) -> Result<impl Responder, Box<dyn Error>> {
    let path = format!("./{}.txt", form.name.as_str());
    let mut file = fs::File::create(path).await?;

    let mut bytes = &form.file.data[..];

    file.write_all(&mut bytes).await?;

    Ok(HttpResponse::Ok())
}

#[utoipa::path(
    context_path = "/service",
    request_body(content = DeviceStartInitRequest, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Ok response with device id and connection params", body = DeviceStartInitResponse)
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
        (status = 200, description = "Ok response")
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
        (status = 200, description = "Ok response with device conf info", body = ObtainDeviceConfInfoResponse)
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
        (status = 200, description = "Ok response")
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
        (status = 200, description = "Ok response")
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

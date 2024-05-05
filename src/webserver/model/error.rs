use core::fmt;

use actix_web::{
    body::BoxBody,
    error::JsonPayloadError,
    error::ResponseError,
    http::{header::TryIntoHeaderValue as _, StatusCode},
};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use utoipa::ToSchema;

use crate::controller;

#[derive(Debug, ToSchema)]
pub struct WebError {
    #[schema(value_type = String, format = Byte)]
    code: StatusCode,
    msg: String,
}

impl WebError {
    pub fn new(code: StatusCode, msg: String) -> Self {
        WebError { code, msg }
    }
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WebError: status: '{}', message: '{}'",
            self.code, self.msg
        )
    }
}

impl Serialize for WebError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("WebError", 2)?;
        state.serialize_field("code", &self.code.as_str())?;
        state.serialize_field("msg", &self.msg)?;
        state.end()
    }
}

impl ResponseError for WebError {
    fn status_code(&self) -> StatusCode {
        self.code
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code());

        match serde_json::to_string(&self) {
            Ok(body) => {
                let mime = mime::APPLICATION_JSON.try_into_value().unwrap();
                res.headers_mut()
                    .insert(actix_web::http::header::CONTENT_TYPE, mime);

                res.set_body(BoxBody::new(body))
            }
            Err(err) => {
                let mime = mime::TEXT_PLAIN_UTF_8.try_into_value().unwrap();
                res.headers_mut()
                    .insert(actix_web::http::header::CONTENT_TYPE, mime);

                res.set_body(BoxBody::new(format!("failed to serialize error: {}", err)))
            }
        }
    }
}

impl From<controller::error::ControllerError> for WebError {
    fn from(value: controller::error::ControllerError) -> Self {
        let (code, msg) = match value {
            controller::error::ControllerError::UnknownDevice(err) => {
                (StatusCode::NOT_FOUND, format!("device not found: {}", err))
            }
            controller::error::ControllerError::IncorrectPayload(err) => {
                (StatusCode::BAD_REQUEST, format!("{}", err))
            }
            controller::error::ControllerError::CommonError(err) => {
                let code = ctrl_err_type_to_actix_code(err.error_type);
                (code, err.msg)
            }
            controller::error::ControllerError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal error".to_string(),
            ),
        };

        WebError { code, msg }
    }
}

impl From<JsonPayloadError> for WebError {
    fn from(value: JsonPayloadError) -> Self {
        Self {
            code: value.status_code(),
            msg: format!("{}", value),
        }
    }
}

impl From<Box<dyn std::error::Error>> for WebError {
    fn from(_: Box<dyn std::error::Error>) -> Self {
        WebError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: "internal error".to_string(),
        }
    }
}

fn ctrl_err_type_to_actix_code(err: controller::error::ErrorType) -> StatusCode {
    match err {
        controller::error::ErrorType::NotFound => StatusCode::NOT_FOUND,
        controller::error::ErrorType::AlreadyExists => StatusCode::CONFLICT,
        controller::error::ErrorType::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        controller::error::ErrorType::IO => StatusCode::INTERNAL_SERVER_ERROR,
        controller::error::ErrorType::Timeout => StatusCode::GATEWAY_TIMEOUT,
        controller::error::ErrorType::InvalidInput => StatusCode::BAD_REQUEST,
        controller::error::ErrorType::FailedPrecondition => StatusCode::BAD_REQUEST,
        controller::error::ErrorType::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

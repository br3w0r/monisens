use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{dev, Result};

use super::super::error::WebError;

pub fn error_parser<B>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let resp = res.response();

    if let Some(err) = resp.error() {
        if let None = err.as_error::<WebError>() {
            let new_err = WebError::new(res.status(), err.to_string());
            let new_resp = res.error_response(new_err);

            return Ok(ErrorHandlerResponse::Response(
                new_resp.map_into_right_body(),
            ));
        }
    }

    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}

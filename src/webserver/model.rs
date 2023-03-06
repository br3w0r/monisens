use crate::controller;

pub struct State {
    pub ctrl: controller::Controller,
}

pub mod contract {
    use serde::Deserialize;
    use actix_multipart::form::{
        MultipartForm,
        bytes::Bytes,
        text::Text,
    };
    use utoipa::ToSchema;


    #[derive(Deserialize)]
    pub struct DeviceStartInitData {
        pub name: String,
    }

    #[derive(Debug, MultipartForm, ToSchema)]
    pub struct TestUploadForm {
        #[schema(value_type = String, format = Binary)]
        #[multipart(rename = "file")]
        pub file: Bytes,
        #[schema(value_type = String, format = Byte)]
        pub name: Text<String>,
    }
}

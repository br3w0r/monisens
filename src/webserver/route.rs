pub mod app {
    use actix_web::{get, HttpRequest};

    #[get("/{_:.*}")]
    pub async fn index(req: HttpRequest) -> String {
        // TODO: serve static
        todo!()
    }
}

pub mod service {
    use std::error::Error;

    use actix_web::{get, post, web::{self, BufMut}, HttpRequest, Responder, Result, HttpResponse};
    use actix_multipart::form::MultipartForm;
    use tokio::{fs, io::{self, AsyncWriteExt}};

    use crate::webserver::model::{contract, State};

    #[utoipa::path(
        context_path = "/service",
        responses(
            (status = 200, description = "Hello, world!", body = String)
        )
    )]
    #[get("/")]
    pub async fn index(data: web::Data<State>) -> String {
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
    pub async fn test_save_files(MultipartForm(form): MultipartForm<contract::TestUploadForm>) -> Result<impl Responder, Box<dyn Error>> {
        let path = format!("./{}.txt", form.name.as_str());
        let mut file = fs::File::create(path).await?;

        let mut bytes = &form.file.data[..];

        file.write_all(&mut bytes).await?;

        Ok(HttpResponse::Ok())
    }

    // #[get("/start-device-init")]
    // pub async fn start_device_init(data: web::Data<State>, req: HttpRequest) -> Result<impl Responder> {
    //     req.headers()

    //     todo!()
    // }
}

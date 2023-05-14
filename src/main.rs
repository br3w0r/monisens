use std::error::Error;
use std::io::Write;

use tokio::runtime::Handle;

mod app;
mod controller;
mod logger;
mod module;
mod query;
mod repo;
mod service;
mod table;
mod tool;
mod webserver;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf = controller::Conf::new()
        .with_repo_dsn("postgres://postgres:pgpass@10.211.55.2:5433/monisens".into());

    let ctrl = controller::Controller::new(conf, Handle::current()).await?;

    println!("Starting web server...");
    std::io::stdout().flush().unwrap();

    webserver::start_server(ctrl).await?;

    Ok(())
}

// fn error_parser<T: Any>(err: T) -> Option<String> {
//     let of_any = &err as &dyn Any;
//     if let Some(sqlx_err) = of_any.downcast_ref::<Error>() {
//         match sqlx_err {
//             Error::Database(db_err) => {
//                 let code: String = match db_err.code() {
//                     Some(c) => c.into_owned(),
//                     _ => String::from("unknown"),
//                 };

//                 Some(format!(
//                     "db error occured: code: '{}'; message: {}",
//                     code,
//                     db_err.message()
//                 ))
//             }

//             Error::RowNotFound => Some(format!("entry was not found")),

//             _ => None,
//         }
//     } else {
//         None
//     }
// }

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

use webserver::config;

const APP_DATA_DIR: &str = "app_data";
const APP_DATA_ENV_KEY: &str = "MONISENS_APP_DATA";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf = controller::Conf::new()
        .with_repo_dsn("postgres://postgres:pgpass@localhost:5433/monisens".into());

    let ctrl = controller::Controller::new(conf, Handle::current()).await?;

    println!("Starting web server...");
    std::io::stdout().flush().unwrap();

    let app_config = init_app_config()?;

    webserver::start_server(ctrl, app_config).await?;

    Ok(())
}

fn init_app_config() -> Result<config::AppConfig, Box<dyn Error>> {
    let exec_dir = get_exec_dir()?;
    let app_data_env_path = std::env::var(APP_DATA_ENV_KEY);
    let app_data = if let Ok(dir) = app_data_env_path {
        dir.parse()
    } else {
        Ok(exec_dir.join(APP_DATA_DIR))
    }?;

    Ok(config::AppConfig::new(app_data))
}

fn get_exec_dir() -> std::io::Result<std::path::PathBuf> {
    let mut exec_dir = std::env::current_exe()?;
    exec_dir.pop();

    Ok(exec_dir)
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

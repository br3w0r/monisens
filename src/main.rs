mod app;
mod logger;
mod module;
mod query;
mod repo;
mod service;
mod table;
mod tool;

use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
use std::error::Error;
use std::{env, ffi::CString};

use query::integration::isqlx as sq;
use query::sqlizer::Sqlizer;
use table::{Field, FieldOption, Table};

#[derive(FromRow, Debug)]
struct Test {
    id: i64,
    #[sqlx(default)]
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let args: Vec<String> = env::args().collect();

    // if args.len() < 2 {
    //     panic!("args.len() < 2");
    // }

    // let mut m = module::Module::new(&args[1])?;
    // let info = m.obtain_device_info()?;

    // println!("{:?}", info);

    // let mut conf = module::DeviceConnectConf::new(vec![
    //     module::ConnParam::new(
    //         "IP".into(),
    //         module::ConnParamValType::String("127.0.0.1".into()),
    //     ),
    //     module::ConnParam::new("Port".into(), module::ConnParamValType::Int(8080)),
    //     module::ConnParam::new(
    //         "Message".into(),
    //         module::ConnParamValType::String("Hello, world!".into()),
    //     ),
    // ]);

    // m.connect_device(&mut conf)?;

    // let conf_info = m.obtain_device_conf_info()?;

    // println!("{:?}", conf_info);

    // let mut conf = vec![
    //     module::DeviceConfEntry::new(1, Some(module::DeviceConfType::Int(5))),
    //     module::DeviceConfEntry::new(2, Some(module::DeviceConfType::ChoiceList(2))),
    //     module::DeviceConfEntry::new(
    //         3,
    //         Some(module::DeviceConfType::String(
    //             CString::new("hello").unwrap(),
    //         )),
    //     ),
    // ];

    // m.configure_device(&mut conf)?;

    // let sensor_infos = m.obtain_sensor_type_infos()?;

    // println!("{:?}", sensor_infos);

    let repo = repo::Repository::new("postgres://postgres:pgpass@localhost:5433/monisens").await?;

    let service = service::Service::new(repo).await?;

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("args.len() < 2");
    }

    let mut file = tokio::fs::File::open(&args[1]).await?;

    let meta = file.metadata().await?;

    let res = service
        .start_device_init("test_device".into(), &mut file)
        .await?;

    println!("{:?}", res);

    println!("Hello!");

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

use std::{env, error::Error};

mod app;
mod controller;
mod logger;
mod module;
mod query;
mod repo;
mod service;
mod table;
mod tool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf = controller::Conf::new()
        .with_repo_dsn("postgres://postgres:pgpass@localhost:5433/monisens".into());

    let ctrl = controller::Controller::new(conf).await?;

    // For example: 1st argument must be a full path to a module
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("args.len() < 2");
    }
    let mut file = tokio::fs::File::open(&args[1]).await?;

    let init_data = ctrl
        .start_device_init("test_device".into(), &mut file)
        .await?;
    println!("{:?}", init_data);

    let conf = vec![
        controller::ConnParam {
            name: "IP".into(),
            value: controller::ConnParamValType::String("127.0.0.1".into()),
        },
        controller::ConnParam {
            name: "Port".into(),
            value: controller::ConnParamValType::Int(8080),
        },
        controller::ConnParam {
            name: "Message".into(),
            value: controller::ConnParamValType::String("Hello, world!".into()),
        },
    ];

    ctrl.connect_device(init_data.id, conf)?;

    let conf_info = ctrl.obtain_device_conf_info(init_data.id);
    println!("{:?}", conf_info);

    let conf = vec![
        controller::DeviceConfEntry {
            id: 1,
            data: Some(controller::DeviceConfType::Int(5)),
        },
        controller::DeviceConfEntry {
            id: 1,
            data: Some(controller::DeviceConfType::ChoiceList(2)),
        },
        controller::DeviceConfEntry {
            id: 3,
            data: Some(controller::DeviceConfType::String("hello".into())),
        },
    ];

    ctrl.configure_device(init_data.id, conf).await?;

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

mod logger;
mod module;
mod query;
mod repo;
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

    let mut id_field = Field::new(1, "id".into(), table::FieldType::Int64).unwrap();
    id_field.add_opt(FieldOption::PrimaryKey).unwrap();
    id_field.add_opt(FieldOption::Unique).unwrap();
    id_field.add_opt(FieldOption::NotNull).unwrap();
    id_field.add_opt(FieldOption::AutoIncrement).unwrap();

    let mut name_field = Field::new(2, "name".into(), table::FieldType::Text).unwrap();
    name_field.add_opt(FieldOption::NotNull).unwrap();

    let mut table = Table::new("test_table".into()).unwrap();
    table.add_field(id_field).unwrap();
    table.add_field(name_field).unwrap();

    repo.create_table(table).await?;

    let mut b = sq::StatementBuilder::new();
    b.table("test_table".to_string())
        .column("name".into())
        .set(vec!["foo".into()])
        .set(vec!["bar".into()]);

    repo.exec(b.insert()).await?;

    let mut b = sq::StatementBuilder::new();
    b.table("test_table".to_string())
        .columns(&["id".into(), "name".into()]);

    let res: Vec<Test> = repo.select(b.select()).await?;

    println!("{:?}", res);

    repo.exec_raw("DROP TABLE test_table").await?;

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

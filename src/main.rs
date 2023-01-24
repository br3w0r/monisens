mod logger;
mod module;
mod query;
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
    println!("Hello, world!");

    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect("postgres://postgres:pgpass@localhost:5433/monisens")
    //     .await?;

    // println!("connected to a pool");

    // let mut b = sq::StatementBuilder::new();
    // b.table("test_parse_table".to_string())
    //     .columns(&["id".into(), "name".into()])
    //     .whereq(sq::gt("id".to_string(), 2));

    // let (sql, args) = b.select().sql()?;

    // let q = sq::query(&sql, &args);
    // let res = q.fetch_all(&pool).await?;
    // let rows: Vec<Test> = res.iter().map(|x| Test::from_row(x).unwrap()).collect();

    // println!("{:?}", rows);

    // let mut b = sq::StatementBuilder::new();
    // b.table("test_parse_table".into());

    // for (i, v) in vec!["this", "is", "a", "test"].drain(0..).enumerate() {
    //     b.set(vec![((i as i32) + 1).into(), v.into()]);
    // }

    // let (sql, args) = b.insert().sql()?;
    // let q = sq::query(&sql, &args);

    // let res = q.execute(&pool).await?;

    // println!("{}", res.rows_affected());

    // let mut b = sq::StatementBuilder::new();
    // b.table("test_parse_table".into())
    //     .whereq(sq::neq("id".into(), 2));

    // let (sql, args) = b.delete().sql()?;
    // let q = sq::query(&sql, &args);

    // let res = q.execute(&pool).await?;

    // println!("{}", res.rows_affected());

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("args.len() < 2");
    }

    let mut m = module::Module::new(&args[1])?;
    let info = m.obtain_device_info()?;

    println!("{:?}", info);

    let mut conf = module::DeviceConnectConf::new(vec![
        module::ConnParam::new(
            "IP".into(),
            module::ConnParamValType::String("127.0.0.1".into()),
        ),
        module::ConnParam::new("Port".into(), module::ConnParamValType::Int(8080)),
        module::ConnParam::new(
            "Message".into(),
            module::ConnParamValType::String("Hello, world!".into()),
        ),
    ]);

    m.connect_device(&mut conf)?;

    let conf_info = m.obtain_device_conf_info()?;

    println!("{:?}", conf_info);

    let mut conf = vec![
        module::DeviceConfEntry::new(1, Some(module::DeviceConfType::Int(5))),
        module::DeviceConfEntry::new(2, Some(module::DeviceConfType::ChoiceList(2))),
        module::DeviceConfEntry::new(
            3,
            Some(module::DeviceConfType::String(
                CString::new("hello").unwrap(),
            )),
        ),
    ];

    m.configure_device(&mut conf)?;

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

mod query;
mod table;
mod tool;

use sqlx::postgres::{PgPoolOptions};
use sqlx::{FromRow};
use std::error::Error;
use std::rc::Rc;

use query::integration::isqlx as sq;
use query::{sqlizer::Sqlizer};
use table::{Field, FieldOption, Table};

#[derive(FromRow, Debug)]
struct Test {
    id: i64,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:pgpass@localhost:5433/monisens")
        .await?;

    println!("connected to a pool");

    let mut b = sq::StatementBuilder::new();
    b.table("test_parse_table".to_string())
        .columns(vec!["id".into(), "name".into()])
        .whereq(sq::eq("id".to_string(), 2));

    let (sql, args) = b.select().sql()?;

    let mut q = sq::query(&sql, &args);
    let mut res = q.fetch_one(&pool).await?;

    let row = Test::from_row(&res);

    println!("{:?}", row);

    Ok(())
}

fn prt(s: &str) {
    print!("{}", s);
}

fn create_test_table(name: String) -> Table {
    let mut id_field = Field::new(1, "id".to_string(), table::FieldType::Int64).unwrap();
    id_field.add_opt(FieldOption::PrimaryKey).unwrap();
    id_field.add_opt(FieldOption::Unique).unwrap();
    id_field.add_opt(FieldOption::NotNull).unwrap();
    id_field.add_opt(FieldOption::AutoIncrement).unwrap();

    let mut name_field = Field::new(2, "name".to_string(), table::FieldType::Text).unwrap();
    name_field.add_opt(FieldOption::NotNull).unwrap();

    let mut table = Table::new(name).unwrap();
    table.add_field(id_field).unwrap();
    table.add_field(name_field).unwrap();

    table
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

mod table;
mod tool;

use sqlx::postgres::PgPoolOptions;
use sqlx::Error;

use table::{Field, FieldOption, Table};

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Hello, world!");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:pgpass@localhost:5433/monisens")
        .await?;

    println!("connected to a pool");

    let table = create_test_table("test_parse_table3".to_string());

    let res = sqlx::query(&table.parse().unwrap())
        .execute(&pool)
        .await?;

    println!("{}", res.rows_affected());

    Ok(())
}

fn create_test_table(name: String) -> Table {
    let mut id_field = Field::new(1, "id".to_string(), table::FieldType::Int64).unwrap();
    id_field.add_opt(FieldOption::PrimaryKey).unwrap();
    id_field.add_opt(FieldOption::NotNull).unwrap();
    id_field.add_opt(FieldOption::AutoIncrement).unwrap();

    let mut name_field = Field::new(2, "name".to_string(), table::FieldType::Text).unwrap();
    name_field.add_opt(FieldOption::NotNull).unwrap();

    let mut table = Table::new(name).unwrap();
    table.add_field(id_field).unwrap();
    table.add_field(name_field).unwrap();

    table
}

fn error_parser(err: &Error) -> Option<String> {
    match err {
        Error::Database(db_err) => {
            let code: String = match db_err.code() {
                Some(c) => c.into_owned(),
                _ => String::from("unknown"),
            };

            Some(format!(
                "db error occured: code: '{}'; message: {}",
                code,
                db_err.message()
            ))
        }

        Error::RowNotFound => Some(format!("entry was not found")),

        _ => None,
    }
}

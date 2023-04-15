use std::vec;

use chrono;
use sqlx::postgres::PgRow;
use sqlx::{Column, FromRow, Row, TypeInfo};

use crate::query::integration::isqlx as sq;
use crate::{
    arg_from_ty, ref_arg_type,
    tool::query_trait::{ColumnsTrait, ValuesTrait},
};
use macros::Table;

use crate::debug_from_display;
use thiserror::Error;

#[derive(sqlx::Type, Debug, PartialEq)]
#[sqlx(type_name = "device_init_state", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceInitState {
    Device,
    Sensors,
}

impl ToString for DeviceInitState {
    fn to_string(&self) -> String {
        match self {
            DeviceInitState::Device => "DEVICE".into(),
            DeviceInitState::Sensors => "SENSORS".into(),
        }
    }
}

ref_arg_type!(DeviceInitState);
arg_from_ty!(DeviceInitState);

#[derive(FromRow, Table)]
pub struct Device {
    #[column]
    pub id: i32,
    #[column]
    pub name: String,
    #[column]
    pub module_dir: String,
    #[column]
    pub data_dir: String,
    #[column]
    pub init_state: DeviceInitState,
}

impl Device {
    pub fn table_name() -> String {
        "device".into()
    }
}

// TODO: macro for this trait
impl ValuesTrait for Device {
    fn values(self, b: &mut crate::query::integration::isqlx::StatementBuilder) {
        b.values(vec![
            self.id.into(),
            self.name.into(),
            self.module_dir.into(),
            self.data_dir.into(),
            self.init_state.into(),
        ]);
    }
}

#[derive(FromRow, Table)]
pub struct DeviceSensor {
    #[column]
    pub device_id: i32,
    #[column]
    pub sensor_table_name: String,
}

impl DeviceSensor {
    pub fn table_name() -> String {
        "device_sensor".into()
    }
}

impl ValuesTrait for DeviceSensor {
    fn values(self, b: &mut crate::query::integration::isqlx::StatementBuilder) {
        b.values(vec![self.device_id.into(), self.sensor_table_name.into()]);
    }
}

/// For retrieving device's sensors data types from `information_schema.columns`
#[derive(FromRow, Table)]
pub struct ColumnType {
    #[column]
    pub table_name: String,
    #[column]
    pub column_name: String,
    #[column]
    pub udt_name: String,
}

pub struct SensorData {
    pub name: String,
    pub data: SensorDataTypeValue,
}

#[derive(Debug, Clone)]
pub enum SensorDataTypeValue {
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Timestamp(chrono::NaiveDateTime),
    String(String),
    JSON(String),
}

impl crate::query::integration::isqlx::ArgType for SensorDataTypeValue {
    fn bind<'q>(
        &'q self,
        q: sqlx::query::Query<
            'q,
            sqlx::postgres::Postgres,
            <sqlx::postgres::Postgres as sqlx::database::HasArguments<'q>>::Arguments,
        >,
    ) -> sqlx::query::Query<
        'q,
        sqlx::postgres::Postgres,
        <sqlx::postgres::Postgres as sqlx::database::HasArguments<'q>>::Arguments,
    > {
        match self {
            SensorDataTypeValue::Int16(v) => v.bind(q),
            SensorDataTypeValue::Int32(v) => v.bind(q),
            SensorDataTypeValue::Int64(v) => v.bind(q),
            SensorDataTypeValue::Float32(v) => v.bind(q),
            SensorDataTypeValue::Float64(v) => v.bind(q),
            SensorDataTypeValue::Timestamp(v) => v.bind(q),
            SensorDataTypeValue::String(v) => v.bind(q),
            SensorDataTypeValue::JSON(v) => v.bind(q),
        }
    }
}

arg_from_ty!(SensorDataTypeValue);

pub struct SensorDataRow(pub Vec<SensorData>);

#[derive(Error)]
pub enum SensorDataDecodeError {
    #[error("static_dir is empty")]
    UnsupportedType(String),
}

debug_from_display!(SensorDataDecodeError);

impl<'r> FromRow<'r, PgRow> for SensorDataRow {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let mut res = Vec::with_capacity(row.len());

        for col in row.columns() {
            let info = col.type_info();
            let data = match info.name() {
                "INT2" => Ok(SensorDataTypeValue::Int16(row.get(col.ordinal()))),
                "INT4" => Ok(SensorDataTypeValue::Int32(row.get(col.ordinal()))),
                "INT8" => Ok(SensorDataTypeValue::Int64(row.get(col.ordinal()))),
                "FLOAT4" => Ok(SensorDataTypeValue::Float32(row.get(col.ordinal()))),
                "FLOAT8" => Ok(SensorDataTypeValue::Float64(row.get(col.ordinal()))),
                "TIMESTAMP" => Ok(SensorDataTypeValue::Timestamp(row.get(col.ordinal()))),
                "TEXT" => Ok(SensorDataTypeValue::String(row.get(col.ordinal()))),
                "JSONB" => Ok(SensorDataTypeValue::JSON(row.get(col.ordinal()))),
                any => Err(sqlx::Error::ColumnDecode {
                    index: "test".into(),
                    source: SensorDataDecodeError::UnsupportedType(any.to_string()).into(),
                }),
            }?;

            res.push(SensorData {
                name: col.name().to_string(),
                data,
            })
        }

        Ok(SensorDataRow(res))
    }
}

#[derive(Default)]
pub struct SensorDataFilter {
    pub from: Option<(String, SensorDataTypeValue)>,
    pub to: Option<(String, SensorDataTypeValue)>,
    pub limit: Option<i32>,
    pub sort: Option<Sort>,
}

impl SensorDataFilter {
    pub fn apply(&self, b: &mut sq::StatementBuilder) {
        if let Some((ref col, ref val)) = self.from {
            b.whereq(sq::gt(col.clone(), val.clone()));
        }

        if let Some((ref col, ref val)) = self.to {
            b.whereq(sq::lt(col.clone(), val.clone()));
        }

        if let Some(ref v) = self.limit {
            b.limit(v.clone());
        }

        if let Some(ref v) = self.sort {
            v.apply(b);
        }
    }
}

pub enum SortOrder {
    ASC,
    DESC,
}

impl ToString for SortOrder {
    fn to_string(&self) -> String {
        match self {
            SortOrder::ASC => "ASC".to_string(),
            SortOrder::DESC => "DESC".to_string(),
        }
    }
}

pub struct Sort {
    pub field: String,
    pub order: SortOrder,
}

impl Sort {
    pub fn apply(&self, b: &mut sq::StatementBuilder) {
        b.order(self.field.clone() + " " + &self.order.to_string());
    }
}

use std::vec;

use sqlx::FromRow;

use crate::{
    arg_from_ty, ref_arg_type,
    tool::query_trait::{ColumnsTrait, ValuesTrait},
};
use macros::Table;

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

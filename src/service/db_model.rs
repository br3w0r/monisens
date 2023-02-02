use sqlx::FromRow;

#[derive(sqlx::Type)]
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

#[derive(FromRow)]
pub struct Device {
    pub id: i32,
    pub name: String,
    pub module_dir: String,
    pub data_dir: String,
    pub init_state: DeviceInitState,
}

impl Device {
    pub fn table_name() -> String {
        "device".into()
    }
}

#[derive(FromRow)]
pub struct DeviceSensor {
    pub device_id: i32,
    pub sensor_table_name: String,
}

impl DeviceSensor {
    pub fn table_name() -> String {
        "device_sensor".into()
    }
}

/// For retrieving device's sensors data types from `information_schema.columns`
#[derive(FromRow)]
pub struct ColumnType {
    pub table_name: String,
    pub column_name: String,
    pub udt_name: String,
}

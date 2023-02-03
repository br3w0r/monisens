use std::collections::HashMap;
use std::fmt;

use crate::debug_from_display;

use super::db_model;

use thiserror::Error;

#[derive(Error)]
pub enum DeviceError {
    // TODO
    #[error("unknown device was provided in device_sensors")]
    DeviceSensorsUnknownDevice,
    #[error("unknown sensor data type in table '{0}' in column '{1}'")]
    SensorDataUnknownType(String, String),
}

debug_from_display!(DeviceError);

enum SensorDataType {
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Timestamp,
    String,
    JSON,
}

impl SensorDataType {
    fn from_db_type(t: &str) -> Option<Self> {
        match t {
            "int2" => Some(Self::Int16),
            "int4" => Some(Self::Int32),
            "int8" => Some(Self::Int64),
            "float4" => Some(Self::Float32),
            "float8" => Some(Self::Float64),
            "timestamp" => Some(Self::Timestamp),
            "text" => Some(Self::String),
            "jsonb" => Some(Self::JSON),
            _ => None,
        }
    }
}

pub struct SensorData {
    name: String,
    typ: SensorDataType,
}

pub struct Sensor {
    /// == sensor's table name
    name: String,

    /// key in the [`HashMap`] is equal to [`SensorData`]`.name`
    data_map: HashMap<String, SensorData>,
}

pub struct Device {
    /// == `device.name` in DB
    name: String,
    module_dir: String,
    data_dir: String,

    /// [`HashMap`]<`sensor's table name`, [`Sensor`]>
    sensor_map: HashMap<String, Sensor>,
}

#[derive(Eq, Hash, PartialEq)]
pub struct DeviceID(i32);

pub struct DeviceManager {
    last_id: DeviceID,
    device_map: HashMap<DeviceID, Device>,
}

impl DeviceManager {
    pub fn new(
        devices: &Vec<db_model::Device>,
        device_sensors: &Vec<db_model::DeviceSensor>,
        sensor_types: &Vec<db_model::ColumnType>,
    ) -> Result<Self, DeviceError> {
        let mut res = Self {
            last_id: DeviceID(0),
            device_map: HashMap::new(),
        };

        // Init devices
        for device in devices {
            res.device_map.insert(
                DeviceID(device.id),
                Device {
                    name: device.name.clone(),
                    module_dir: device.module_dir.clone(),
                    data_dir: device.data_dir.clone(),
                    sensor_map: HashMap::new(),
                },
            );

            if device.id > res.last_id.0 {
                res.last_id = DeviceID(device.id);
            }
        }

        // Init all sensors
        let mut sensors_res: HashMap<String, Sensor> = HashMap::new();

        for sensor_type in sensor_types {
            let sensor = sensors_res
                .entry(sensor_type.table_name.clone())
                .or_insert(Sensor {
                    name: sensor_type.table_name.clone(),
                    data_map: HashMap::new(),
                });

            if let Some(typ) = SensorDataType::from_db_type(&sensor_type.udt_name) {
                sensor.data_map.insert(
                    sensor_type.column_name.clone(),
                    SensorData {
                        name: sensor_type.column_name.clone(),
                        typ: typ,
                    },
                );
            } else {
                return Err(DeviceError::SensorDataUnknownType(
                    sensor_type.table_name.clone(),
                    sensor_type.column_name.clone(),
                ));
            }
        }

        // Map sensors to its devices
        for device_sensor in device_sensors {
            let device_id = DeviceID(device_sensor.device_id);

            if let Some(device) = res.device_map.get_mut(&device_id) {
                if let Some(sensor) = sensors_res.remove(&device_sensor.sensor_table_name) {
                    device
                        .sensor_map
                        .insert(device_sensor.sensor_table_name.clone(), sensor);
                }
            } else {
                return Err(DeviceError::DeviceSensorsUnknownDevice);
            }
        }

        Ok(res)
    }

    pub fn device_count(&self) -> usize {
        self.device_map.len()
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self {
            last_id: DeviceID(0),
            device_map: Default::default(),
        }
    }
}

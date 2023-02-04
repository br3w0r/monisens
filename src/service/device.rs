use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::sync::RwLock;

use tokio::fs;
use tokio::io;
use tokio::io::AsyncRead;

use crate::{app, debug_from_display};

use super::db_model;

#[cfg(target_os = "macos")]
const MODULE_FILE_EXT: &str = ".dylib";

#[cfg(target_os = "linux")]
const MODULE_FILE_EXT: &str = ".so";

#[cfg(target_os = "windows")]
const MODULE_FILE_EXT: &str = ".dll";

#[derive(thiserror::Error)]
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

#[derive(Debug, Eq, Hash, PartialEq, Default, Clone, Copy)]
pub struct DeviceID(i32);

impl From<DeviceID> for i32 {
    fn from(value: DeviceID) -> Self {
        value.0
    }
}

pub struct DeviceManager {
    last_id: Arc<AtomicI32>,
    device_map: Arc<RwLock<HashMap<DeviceID, Arc<RwLock<Device>>>>>,
    data_dir: Arc<String>,
}

impl DeviceManager {
    /// `new` method creates an internal map of devices based on the provided `devices` vector and associates
    /// the device with its sensors based on the information in `device_sensors` and `sensor_types`.
    pub fn new(
        devices: &Vec<db_model::Device>,
        device_sensors: &Vec<db_model::DeviceSensor>,
        sensor_types: &Vec<db_model::ColumnType>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut last_id: i32 = 0;
        let mut device_map: HashMap<DeviceID, Arc<RwLock<Device>>> = Default::default();

        // Init devices
        for device in devices {
            device_map.insert(
                DeviceID(device.id),
                Arc::new(RwLock::new(Device {
                    name: device.name.clone(),
                    module_dir: device.module_dir.clone(),
                    data_dir: device.data_dir.clone(),
                    sensor_map: HashMap::new(),
                })),
            );

            if device.id > last_id {
                last_id = device.id;
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
                return Err(Box::new(DeviceError::SensorDataUnknownType(
                    sensor_type.table_name.clone(),
                    sensor_type.column_name.clone(),
                )));
            }
        }

        // Map sensors to its devices
        for device_sensor in device_sensors {
            let device_id = DeviceID(device_sensor.device_id);

            if let Some(device) = device_map.get(&device_id) {
                if let Some(sensor) = sensors_res.remove(&device_sensor.sensor_table_name) {
                    let mut device = device.write().unwrap();

                    device
                        .sensor_map
                        .insert(device_sensor.sensor_table_name.clone(), sensor);
                }
            } else {
                return Err(Box::new(DeviceError::DeviceSensorsUnknownDevice));
            }
        }

        Ok(Self {
            last_id: Arc::new(AtomicI32::new(last_id)),
            device_map: Arc::new(RwLock::new(device_map)),
            data_dir: Arc::new(check_and_return_base_dir()),
        })
    }

    /// `start_device_init` creates directories for device's data and module and writes
    /// module file to `<app_dir>/device/<id>-<device_name_snake_case>/module/` directory
    ///
    /// Created structure:
    /// ```
    /// <app_dir>/
    ///     device/
    ///         <id>-<device_name_snake_case>/
    ///             module/
    ///             data/
    /// ```
    pub async fn start_device_init<'f, F>(
        &self,
        name: String,
        module_file: &'f mut F,
    ) -> Result<(DeviceID, String, String), Box<dyn Error>>
    where
        F: AsyncRead + Unpin + ?Sized,
    {
        let id = self.inc_last_id();

        let dir_name = id.0.to_string() + "-" + &name + "/";
        self.create_data_dir(&dir_name).await?;

        let module_dir = dir_name.clone() + "module/";
        self.create_data_dir(&module_dir).await?;

        let data_dir = dir_name.clone() + "data/";
        self.create_data_dir(&data_dir).await?;

        let full_module_path = (*self.data_dir).clone() + &module_dir + "lib" + MODULE_FILE_EXT;
        create_file(&full_module_path, module_file).await?;

        let device = Device {
            name,
            module_dir: module_dir.clone(),
            data_dir: data_dir.clone(),
            sensor_map: Default::default(),
        };

        (*self.device_map.write().unwrap()).insert(id, Arc::new(RwLock::new(device)));

        Ok((id, module_dir, data_dir))
    }

    fn inc_last_id(&self) -> DeviceID {
        let prev_last_id = self.last_id.fetch_add(1, Ordering::SeqCst);

        DeviceID(prev_last_id + 1)
    }

    async fn create_data_dir(&self, rel_path: &str) -> io::Result<()> {
        fs::create_dir((*self.data_dir).clone() + rel_path).await
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self {
            last_id: Default::default(),
            device_map: Default::default(),
            data_dir: Arc::new(check_and_return_base_dir()),
        }
    }
}

fn check_and_return_base_dir() -> String {
    let path = app::data_dir() + "device/";
    let p = Path::new(&path);

    if !p.is_dir() {
        std::fs::create_dir(p).unwrap();
    }

    path
}

async fn create_file<'a, R: AsyncRead + Unpin + ?Sized>(
    path: &str,
    data: &'a mut R,
) -> io::Result<()> {
    if let Ok(_) = fs::File::open(path).await {
        return Err(io::ErrorKind::AlreadyExists.into());
    }

    let mut file = fs::File::create(path).await?;
    io::copy(data, &mut file).await?;

    Ok(())
}

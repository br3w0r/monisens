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

use crate::{app, debug_from_display, table::FieldType};

use super::{db_model, model};

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
    #[error("device with id '{0}' was not found. Most probably it was deleted")]
    DeviceNotFound(DeviceID),
    #[error("device with id '{0}' was not configured")]
    DeviceNotConfigured(DeviceID),
}

debug_from_display!(DeviceError);

#[derive(Clone)]
pub enum SensorDataType {
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

    pub fn to_table_type(&self) -> FieldType {
        match self {
            SensorDataType::Int16 => FieldType::Int16,
            SensorDataType::Int32 => FieldType::Int32,
            SensorDataType::Int64 => FieldType::Int64,
            SensorDataType::Float32 => FieldType::Float32,
            SensorDataType::Float64 => FieldType::Float64,
            SensorDataType::Timestamp => FieldType::Timestamp,
            SensorDataType::String => FieldType::Text,
            SensorDataType::JSON => FieldType::JSON,
        }
    }
}

#[derive(Clone)]
pub struct SensorDataEntry {
    pub name: String,
    pub typ: SensorDataType,
}

pub struct Sensor {
    /// == sensor's table name // iss-96: this is not true. It's a human-readable name
    pub name: String,

    /// key in the [`HashMap`] is equal to [`SensorData`]`.name`
    pub data_map: HashMap<String, SensorDataEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceInitState {
    Device,
    Sensors,
}

impl From<&db_model::DeviceInitState> for DeviceInitState {
    fn from(value: &db_model::DeviceInitState) -> Self {
        match value {
            db_model::DeviceInitState::Device => DeviceInitState::Device,
            db_model::DeviceInitState::Sensors => DeviceInitState::Sensors,
        }
    }
}

pub struct Device {
    /// == `device.name` in DB
    name: String,
    module_dir: String,
    data_dir: String,
    init_state: DeviceInitState,

    /// [`HashMap`]<`sensor's table name`, [`Sensor`]>
    sensor_map: HashMap<String, Sensor>,
}

impl Device {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_sensors(&self) -> &HashMap<String, Sensor> {
        &self.sensor_map
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Default, Clone, Copy)]
pub struct DeviceID(i32);

impl DeviceID {
    pub fn get_raw(&self) -> i32 {
        self.0
    }
}

impl From<DeviceID> for i32 {
    fn from(value: DeviceID) -> Self {
        value.0
    }
}

impl fmt::Display for DeviceID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// DeviceInfo contains basic info about device that may be read by user
pub struct DeviceInfo {
    pub id: DeviceID,
    pub name: String,
}

/// SensorInfo contains basic info about device's sensors that may be read by user
pub struct SensorInfo {
    pub name: String,
    pub data: Vec<SensorDataEntry>,
}

/// `DeviceManager` hosts data of all devices like names and data folders, sensors info etc.
#[derive(Clone)]
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
        let mut device_map: HashMap<DeviceID, Arc<RwLock<Device>>> =
            HashMap::with_capacity(devices.len());

        // Init devices
        for device in devices {
            device_map.insert(
                DeviceID(device.id),
                Arc::new(RwLock::new(Device {
                    name: device.name.clone(),
                    module_dir: device.module_dir.clone(),
                    data_dir: device.data_dir.clone(),
                    sensor_map: HashMap::new(),
                    init_state: (&device.init_state).into(),
                })),
            );

            if device.id > last_id {
                last_id = device.id;
            }
        }

        // Init all sensors
        let mut sensors_res: HashMap<String, Sensor> = HashMap::with_capacity(device_sensors.len());

        for sensor_type in sensor_types {
            let sensor = sensors_res
                .entry(sensor_type.table_name.clone())
                .or_insert(Sensor {
                    name: sensor_type.table_name.clone(),
                    data_map: HashMap::new(),
                });

            let typ = SensorDataType::from_db_type(&sensor_type.udt_name).ok_or(
                DeviceError::SensorDataUnknownType(
                    sensor_type.table_name.clone(),
                    sensor_type.column_name.clone(),
                ),
            )?;

            sensor.data_map.insert(
                sensor_type.column_name.clone(),
                SensorDataEntry {
                    name: sensor_type.column_name.clone(),
                    typ: typ,
                },
            );
        }

        // Map sensors to its devices
        for device_sensor in device_sensors {
            let device_id = DeviceID(device_sensor.device_id);

            let device = device_map
                .get(&device_id)
                .ok_or(DeviceError::DeviceSensorsUnknownDevice)?;

            if let Some(sensor) = sensors_res.remove(&device_sensor.sensor_table_name) {
                let mut device = device.write().unwrap();

                device
                    .sensor_map
                    .insert(device_sensor.sensor_table_name.clone(), sensor);
            }
        }

        let res = Self {
            last_id: Arc::new(AtomicI32::new(last_id)),
            device_map: Arc::new(RwLock::new(device_map)),
            data_dir: Arc::new(check_and_return_base_dir()),
        };

        // TODO: Replace with logger
        println!("Inited DeviceManager with data_dir = {}", &res.data_dir);

        Ok(res)
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
    ) -> Result<model::DeviceInitData, Box<dyn Error>>
    where
        F: AsyncRead + Unpin + ?Sized,
    {
        let id = self.inc_last_id();

        let dir_name = build_device_dir_name(&id, &name);
        self.create_data_dir(&dir_name).await?;

        let module_dir = dir_name.clone() + "module/";
        self.create_data_dir(&module_dir).await?;

        let data_dir = dir_name.clone() + "data/";
        self.create_data_dir(&data_dir).await?;

        let full_module_path = self.full_module_file_path(&module_dir);
        create_file(&full_module_path, module_file).await?;

        let device = Device {
            name,
            module_dir: module_dir.clone(),
            data_dir: data_dir.clone(),
            sensor_map: Default::default(),
            init_state: DeviceInitState::Device,
        };

        (*self.device_map.write().unwrap()).insert(id, Arc::new(RwLock::new(device)));

        Ok(model::DeviceInitData {
            id,
            module_file: full_module_path,
            data_dir,
            module_dir,
            init_state: DeviceInitState::Device,
        })
    }

    pub fn device_sensor_init(
        &self,
        device_id: &DeviceID,
        sensors: Vec<Sensor>,
    ) -> Result<(), DeviceError> {
        let device = self.get_device(device_id)?;
        let mut device = device.write().unwrap();
        for sensor in sensors {
            device.sensor_map.insert(sensor.name.clone(), sensor);
        }
        device.init_state = DeviceInitState::Sensors;

        Ok(())
    }

    pub fn get_device_name(&self, id: &DeviceID) -> Result<String, DeviceError> {
        let device = self.get_device(id)?;
        let device = device.read().unwrap();

        Ok(device.name.clone())
    }

    pub fn get_device_init_state(&self, id: DeviceID) -> Result<DeviceInitState, DeviceError> {
        let device = self.get_device(&id)?;
        let device = device.read().unwrap();

        Ok(device.init_state.clone())
    }

    pub async fn delete_device(&self, id: &DeviceID) -> Result<(), Box<dyn Error>> {
        let mut device_map = self.device_map.write().unwrap();

        let device = device_map
            .get(id)
            .ok_or(DeviceError::DeviceNotFound(id.clone()))?
            .clone();

        // Intentionally lock device for write 'cause we're deleting it
        let device = device.write().unwrap();

        let device_dir = self.data_dir.to_string() + &build_device_dir_name(id, &device.name);
        fs::remove_dir_all(device_dir).await?;

        device_map.remove(id);

        Ok(())
    }

    pub fn get_device_ids(&self) -> Vec<DeviceID> {
        self.device_map.read().unwrap().keys().copied().collect()
    }

    pub fn get_init_data_all_devices(&self) -> Vec<model::DeviceInitData> {
        let device_map = self.device_map.read().unwrap();
        let mut res = Vec::with_capacity(device_map.len());
        for (id, data_handler) in device_map.iter() {
            let data = data_handler.read().unwrap();

            res.push(model::DeviceInitData {
                id: id.clone(),
                module_dir: data.module_dir.clone(),
                data_dir: self.full_data_dir(&data.data_dir),
                module_file: self.full_module_file_path(&data.module_dir),
                init_state: data.init_state.clone(),
            })
        }

        res
    }

    /// get_device_info_list returns an unsorted list of devices.
    ///
    /// Devices must be configured to be returned (`init_state == DeviceInitState::Sensors`)
    pub fn get_device_info_list(&self) -> Vec<DeviceInfo> {
        let device_map = self.device_map.read().unwrap();
        let mut res = Vec::with_capacity(device_map.len());
        for (id, data_handler) in device_map.iter() {
            let data = data_handler.read().unwrap();

            if data.init_state == DeviceInitState::Sensors {
                res.push(DeviceInfo {
                    id: id.clone(),
                    name: data.get_name().clone(),
                })
            }
        }

        res
    }

    /// get_device_info_list returns list of device's sensors and their data types.
    ///
    /// Both sensors and their data types are not sorted.
    ///
    /// If the device is not configured, an error `DeviceError::DeviceNotConfigured` is returned.
    pub fn get_device_sensor_info(
        &self,
        device_id: DeviceID,
    ) -> Result<Vec<SensorInfo>, DeviceError> {
        let device = self.get_device(&device_id)?;
        let device = device.read().unwrap();

        if device.init_state != DeviceInitState::Sensors {
            return Err(DeviceError::DeviceNotConfigured(device_id));
        }

        Ok(device
            .sensor_map
            .iter()
            .map(|(name, sensor)| SensorInfo {
                name: name.clone(),
                data: sensor
                    .data_map
                    .iter()
                    .map(|(name, data)| SensorDataEntry {
                        name: name.clone(),
                        typ: data.typ.clone(),
                    })
                    .collect(),
            })
            .collect())
    }

    fn inc_last_id(&self) -> DeviceID {
        let prev_last_id = self.last_id.fetch_add(1, Ordering::SeqCst);

        DeviceID(prev_last_id + 1)
    }

    fn get_device(&self, id: &DeviceID) -> Result<Arc<RwLock<Device>>, DeviceError> {
        if let Some(device) = self.device_map.read().unwrap().get(id) {
            Ok(device.clone())
        } else {
            Err(DeviceError::DeviceNotFound(id.clone()))
        }
    }

    async fn create_data_dir(&self, rel_path: &str) -> io::Result<()> {
        fs::create_dir(self.full_data_dir(rel_path)).await
    }

    fn full_data_dir(&self, data_dir: &str) -> String {
        (*self.data_dir).clone() + data_dir
    }

    fn full_module_file_path(&self, module_dir: &str) -> String {
        (*self.data_dir).clone() + &module_dir + "lib" + MODULE_FILE_EXT
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

fn build_device_dir_name(id: &DeviceID, name: &String) -> String {
    id.0.to_string() + "-" + &name + "/"
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

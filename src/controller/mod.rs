mod conf;
mod error;
mod model;

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    sync::{Arc, Mutex, RwLock},
};

use tokio::io::AsyncRead;

use crate::{module, repo, service};

use model::internal::*;

pub use conf::*;
pub use error::*;
pub use model::*;

#[derive(Clone)]
pub struct Controller {
    svc: service::Service,
    devices: Arc<RwLock<HashMap<i32, Arc<Mutex<Device>>>>>,
}

impl Controller {
    pub async fn new(conf: Conf) -> Result<Self, Box<dyn Error>> {
        let repo = repo::Repository::new(conf.get_repo_dsn()).await?;
        let svc = service::Service::new(repo).await?;

        let device_init_datas = svc.get_init_data_all_devices();
        let mut mods = HashMap::with_capacity(device_init_datas.len());

        for data in device_init_datas {
            let m = module::Module::new(&data.module_file)?;
            mods.insert(
                data.id.get_raw(),
                Arc::new(Mutex::new(Device {
                    id: data.id,
                    module: m,
                })),
            );
        }

        Ok(Self {
            svc,
            devices: Arc::new(RwLock::new(mods)),
        })
    }

    pub async fn start_device_init<'f, F: AsyncRead + Unpin + ?Sized>(
        &self,
        name: String,
        module_file: &'f mut F,
    ) -> Result<DeviceInitData, Box<dyn Error>> {
        let device_init_data = self.svc.start_device_init(name, module_file).await?;

        let res = {
            let mut m = module::Module::new(&device_init_data.module_file)?;
            let mut device_info = m.obtain_device_info()?;

            self.devices.write().unwrap().insert(
                device_init_data.id.get_raw(),
                Arc::new(Mutex::new(Device {
                    id: device_init_data.id.clone(),
                    module: m,
                })),
            );

            Ok(DeviceInitData {
                id: device_init_data.id.get_raw(),
                conn_params: device_info.drain(..).map(|v| v.into()).collect(),
            })
        };

        if let Err(_) = res {
            // TODO: logging?
            self.svc.interrupt_device_init(device_init_data.id).await?;
        }

        res
    }

    pub fn connect_device(&self, id: i32, conf: DeviceConnectConf) -> Result<(), Box<dyn Error>> {
        let device_lock = self.get_device(&id)?;
        let mut device = device_lock.lock().unwrap();

        let mut conn_conf = conf.into();

        device.module.connect_device(&mut conn_conf)?;

        Ok(())
    }

    pub fn obtain_device_conf_info(&self, id: i32) -> Result<DeviceConfInfo, Box<dyn Error>> {
        let device_lock = self.get_device(&id)?;
        let mut device = device_lock.lock().unwrap();

        let device_conf_info = device.module.obtain_device_conf_info()?;

        Ok(device_conf_info.into())
    }

    pub async fn configure_device(
        &self,
        id: i32,
        mut confs: Vec<DeviceConfEntry>,
    ) -> Result<(), Box<dyn Error>> {
        let device_lock = self.get_device(&id)?;
        let mut device = device_lock.lock().unwrap();

        let mut device_conf = confs.drain(..).map(|v| v.into()).collect();

        device.module.configure_device(&mut device_conf)?;

        let sensor = service_sensor_from_module(device.module.obtain_sensor_type_infos()?);

        self.svc.device_sensor_init(device.id, sensor).await?;

        Ok(())
    }

    pub async fn interrupt_device_init(&self, id: i32) -> Result<(), Box<dyn Error>> {
        let device_lock = self.get_device(&id)?;
        let device = device_lock.lock().unwrap();

        self.svc.interrupt_device_init(device.id).await?;

        self.devices.write().unwrap().remove(&id);

        Ok(())
    }

    fn get_device(&self, id: &i32) -> Result<Arc<Mutex<Device>>, ControllerError> {
        self.devices
            .read()
            .unwrap()
            .get(id)
            .ok_or(ControllerError::UnknownDevice(id.clone()))
            .cloned()
    }
}

fn service_sensor_from_module(
    mut sensor_type_infos: Vec<module::SensorTypeInfo>,
) -> Vec<service::Sensor> {
    sensor_type_infos
        .drain(..)
        .map(|mut sensor| {
            let mut data_map = HashMap::with_capacity(sensor.data_type_infos.len());

            for data_type_info in sensor.data_type_infos {
                data_map.insert(
                    data_type_info.name.clone(),
                    service::SensorData {
                        name: data_type_info.name,
                        typ: service_sensor_data_type_from_module(data_type_info.typ),
                    },
                );
            }

            service::Sensor {
                name: sensor.name,
                data_map: data_map,
            }
        })
        .collect()
}

fn service_sensor_data_type_from_module(
    data_type: module::SensorDataType,
) -> service::SensorDataType {
    match data_type {
        module::SensorDataType::Int16 => service::SensorDataType::Int16,
        module::SensorDataType::Int32 => service::SensorDataType::Int32,
        module::SensorDataType::Int64 => service::SensorDataType::Int64,
        module::SensorDataType::Float32 => service::SensorDataType::Float32,
        module::SensorDataType::Float64 => service::SensorDataType::Float64,
        module::SensorDataType::Timestamp => service::SensorDataType::Timestamp,
        module::SensorDataType::String => service::SensorDataType::String,
        module::SensorDataType::JSON => service::SensorDataType::JSON,
    }
}

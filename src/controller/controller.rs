use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex, RwLock},
};

use tokio::{io::AsyncRead, runtime::Handle};

use crate::module;

use super::error::*;
use super::interface::service;
use super::model::internal::*;
use super::model::*;
use super::msg;

#[derive(Clone)]
pub struct Controller<S: service::IService> {
    svc: S,
    tokio_handle: Handle,
    devices: Arc<RwLock<HashMap<i32, Arc<Mutex<Device<S>>>>>>,
}

impl<S: service::IService + 'static> Controller<S> {
    pub async fn new(tokio_handle: Handle, svc: S) -> Result<Self, Box<dyn Error>> {
        let device_init_datas = svc.get_init_data_all_devices();
        let mut mods = HashMap::with_capacity(device_init_datas.len());

        for data in device_init_datas {
            let m = module::Module::new(&data.module_file, &data.full_data_dir).map_err(|err| {
                format!(
                    "failed to init module; file: {}; err: {err}",
                    data.module_file.to_str().unwrap_or("[invalid file path]")
                )
            })?;
            let device = Arc::new(Mutex::new(Device {
                id: data.id,
                module: m,
                msg_handler: None,
            }));

            if data.init_state == DeviceInitState::Sensors {
                let mut device = device.lock().unwrap();
                let msg_handler = msg::Handler::new(data.id, svc.clone(), tokio_handle.clone());

                device
                    .module
                    .start(msg_handler.clone())
                    .map_err(|err| format!("failed to start device: {err}"))?;

                device.msg_handler = Some(msg_handler);
            }

            mods.insert(data.id.get_raw(), device);
        }

        Ok(Self {
            svc,
            tokio_handle,
            devices: Arc::new(RwLock::new(mods)),
        })
    }

    pub async fn start_device_init<'f, F: AsyncRead + Unpin + ?Sized>(
        &self,
        name: String,
        module_file: &'f mut F,
    ) -> Result<DeviceConnData, Box<dyn Error>> {
        let device_init_data = self.svc.start_device_init(name, module_file).await?;

        let res = {
            let mut m = module::Module::new(
                &device_init_data.module_file,
                &device_init_data.full_data_dir,
            )?;
            let mut device_info = m.obtain_device_info()?;

            self.devices.write().unwrap().insert(
                device_init_data.id.get_raw(),
                Arc::new(Mutex::new(Device {
                    id: device_init_data.id.clone(),
                    module: m,
                    msg_handler: None,
                })),
            );

            Ok(DeviceConnData {
                id: device_init_data.id.clone(),
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
        {
            let device_lock = self.get_device(&id)?;
            let mut device = device_lock.lock().unwrap();

            let mut device_conf = confs.drain(..).map(|v| v.into()).collect();

            device.module.configure_device(&mut device_conf)?;

            let sensor_infos = device.module.obtain_sensor_type_infos()?;
            let sensor = service_sensor_from_module(sensor_infos);

            self.svc.device_sensor_init(device.id, sensor).await?;
        }

        // Start receiving data from device's sensors
        self.start_device(id)?;

        Ok(())
    }

    pub async fn interrupt_device_init(&self, id: i32) -> Result<(), Box<dyn Error>> {
        let device_lock = self.get_device(&id)?;
        let device = device_lock.lock().unwrap();

        self.svc.interrupt_device_init(device.id).await?;

        self.devices.write().unwrap().remove(&id);

        Ok(())
    }

    fn start_device(&self, id: i32) -> Result<(), Box<dyn Error>> {
        let device_lock = self.get_device(&id)?;
        let mut device = device_lock.lock().unwrap();

        // Get device's handler (with lazy loading)
        let msg_handler = if let Some(ref msg_handler) = device.msg_handler {
            msg_handler.clone()
        } else {
            let msg_handler =
                msg::Handler::new(device.id, self.svc.clone(), self.tokio_handle.clone());
            device.msg_handler = Some(msg_handler.clone());

            msg_handler
        };

        device.module.start(msg_handler.clone())?;

        Ok(())
    }

    fn stop_device(&self, id: i32) -> Result<(), Box<dyn Error>> {
        let device_lock = self.get_device(&id)?;
        let mut device = device_lock.lock().unwrap();

        device.module.stop()?;

        Ok(())
    }

    pub async fn get_sensor_data(
        &self,
        data: GetSensorDataPayload,
    ) -> Result<GetSensorDataResult, Box<dyn Error>> {
        if data.fields.len() == 0 {
            return Err(ControllerError::IncorrectPayload("data.fields is empty".into()).into());
        }

        let device_id = self.get_device_id(&data.device_id)?;

        let res = self
            .svc
            .get_sensor_data(
                device_id,
                data.sensor.clone(),
                data.fields.clone(),
                data.to_sensor_data_filter(),
            )
            .await?;

        Ok(sensor_data_result_from_service(res))
    }

    pub fn get_device_info_list(&self) -> Vec<DeviceInfo> {
        self.svc.get_device_info_list()
    }

    pub fn get_device_sensor_info(
        &self,
        device_id: i32,
    ) -> Result<Vec<SensorInfo>, Box<dyn Error>> {
        let device_id = self.get_device_id(&device_id)?;

        self.svc.get_device_sensor_info(device_id)
    }

    pub async fn save_monitor_conf(
        &self,
        monitor_conf: MonitorConf,
    ) -> Result<i32, Box<dyn Error>> {
        self.svc.save_monitor_conf(monitor_conf).await
    }

    pub async fn get_monitor_conf_list(
        &self,
        filter: MonitorConfListFilter,
    ) -> Result<Vec<MonitorConf>, Box<dyn Error>> {
        self.svc.get_monitor_conf_list(filter).await
    }

    fn get_device(&self, id: &i32) -> Result<Arc<Mutex<Device<S>>>, ControllerError> {
        self.devices
            .read()
            .unwrap()
            .get(id)
            .ok_or(ControllerError::UnknownDevice(id.clone()))
            .cloned()
    }

    fn get_device_id(&self, id: &i32) -> Result<DeviceID, ControllerError> {
        let device_lock = self.get_device(id)?;
        let device = device_lock.lock().unwrap();

        Ok(device.id)
    }
}

fn service_sensor_from_module(mut sensor_type_infos: Vec<module::SensorTypeInfo>) -> Vec<Sensor> {
    sensor_type_infos
        .drain(..)
        .map(|sensor| {
            let mut data_map = HashMap::with_capacity(sensor.data_type_infos.len());

            for data_type_info in sensor.data_type_infos {
                data_map.insert(
                    data_type_info.name.clone(),
                    SensorDataEntry {
                        name: data_type_info.name,
                        typ: service_sensor_data_type_from_module(data_type_info.typ),
                    },
                );
            }

            Sensor {
                name: sensor.name,
                data_map: data_map,
            }
        })
        .collect()
}

fn service_sensor_data_type_from_module(data_type: module::SensorDataType) -> SensorDataType {
    match data_type {
        module::SensorDataType::Int16 => SensorDataType::Int16,
        module::SensorDataType::Int32 => SensorDataType::Int32,
        module::SensorDataType::Int64 => SensorDataType::Int64,
        module::SensorDataType::Float32 => SensorDataType::Float32,
        module::SensorDataType::Float64 => SensorDataType::Float64,
        module::SensorDataType::Timestamp => SensorDataType::Timestamp,
        module::SensorDataType::String => SensorDataType::String,
        module::SensorDataType::JSON => SensorDataType::JSON,
    }
}

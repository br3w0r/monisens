use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex, RwLock},
};

use tokio::{io::AsyncRead, runtime::Handle};

use crate::logger;
use crate::{kv_any, kv_val, kvs};

use super::error::*;
use super::interface::{
    module::{IModule, IModuleFactory},
    service::IService,
};
use super::model::internal::*;
use super::model::*;
use super::msg;

pub struct Controller<S: IService, M: IModule, MF: IModuleFactory<M>> {
    _module_factory: std::marker::PhantomData<MF>,
    svc: S,
    tokio_handle: Handle,
    devices: Arc<RwLock<HashMap<i32, Arc<Mutex<Device<S, M>>>>>>,
}

impl<S: IService + 'static, M: IModule + 'static, MF: IModuleFactory<M>> Controller<S, M, MF> {
    pub async fn new(tokio_handle: Handle, svc: S) -> Result<Self, ControllerError> {
        let device_init_datas = svc.get_init_data_all_devices()?;
        let mut mods = HashMap::with_capacity(device_init_datas.len());

        for data in device_init_datas {
            let m = MF::create_module(&data.module_file, &data.full_data_dir)?;
            let device = Arc::new(Mutex::new(Device {
                id: data.id,
                module: m,
                msg_handler: None,
            }));

            if data.init_state == DeviceInitState::Sensors {
                let mut device = device.lock().unwrap();
                let msg_handler = msg::Handler::new(data.id, svc.clone(), tokio_handle.clone());

                device.module.start(msg_handler.clone())?;

                device.msg_handler = Some(msg_handler);
            }

            mods.insert(data.id.get_raw(), device);
        }

        Ok(Self {
            _module_factory: std::marker::PhantomData,
            svc,
            tokio_handle,
            devices: Arc::new(RwLock::new(mods)),
        })
    }

    pub async fn start_device_init<'f, F: AsyncRead + Unpin + ?Sized>(
        &self,
        name: String,
        module_file: &'f mut F,
    ) -> Result<DeviceConnData, ControllerError> {
        let device_init_data = self
            .svc
            .start_device_init(name.clone(), module_file)
            .await?;

        let res = self.init_device(
            &device_init_data.module_file,
            &device_init_data.full_data_dir,
            device_init_data.id.clone(),
        );

        if let Err(ref err) = res {
            logger::error_kv(
                "failed to init device",
                kvs!("name" => kv_any!(&name), "error" => kv_val!(err)),
            );
            self.svc.interrupt_device_init(device_init_data.id).await?;
        }

        res
    }

    pub fn connect_device(&self, id: i32, conf: DeviceConnectConf) -> Result<(), ControllerError> {
        let device_lock = self.get_device(&id)?;
        let mut device = device_lock.lock().unwrap();

        let conn_conf = conf.into();

        device.module.connect_device(conn_conf)?;

        Ok(())
    }

    pub fn obtain_device_conf_info(&self, id: i32) -> Result<DeviceConfInfo, ControllerError> {
        let device_lock = self.get_device(&id)?;
        let mut device = device_lock.lock().unwrap();

        let device_conf_info = device.module.obtain_device_conf_info()?;

        Ok(device_conf_info.into())
    }

    pub async fn configure_device(
        &self,
        id: i32,
        confs: Vec<DeviceConfEntry>,
    ) -> Result<(), ControllerError> {
        {
            let device_lock = self.get_device(&id)?;
            let mut device = device_lock.lock().unwrap();

            device.module.configure_device(confs)?;

            let sensor_infos = device.module.obtain_sensor_type_infos()?;

            self.svc.device_sensor_init(device.id, sensor_infos).await?;
        }

        // Start receiving data from device's sensors
        self.start_device(id)?;

        Ok(())
    }

    pub async fn interrupt_device_init(&self, id: i32) -> Result<(), ControllerError> {
        let device_lock = self.get_device(&id)?;
        let device = device_lock.lock().unwrap();

        self.svc.interrupt_device_init(device.id).await?;

        self.devices.write().unwrap().remove(&id);

        Ok(())
    }

    fn init_device<P: AsRef<Path>>(
        &self,
        mod_path: P,
        data_dir: P,
        device_id: DeviceID,
    ) -> Result<DeviceConnData, ControllerError> {
        let mut m = MF::create_module(mod_path, data_dir)?;
        let device_info = m.obtain_device_conn_info()?;

        self.devices.write().unwrap().insert(
            device_id.get_raw(),
            Arc::new(Mutex::new(Device {
                id: device_id.clone(),
                module: m,
                msg_handler: None,
            })),
        );

        Ok(DeviceConnData {
            id: device_id,
            conn_params: device_info,
        })
    }

    fn start_device(&self, id: i32) -> Result<(), ControllerError> {
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

    fn stop_device(&self, id: i32) -> Result<(), ControllerError> {
        let device_lock = self.get_device(&id)?;
        let mut device = device_lock.lock().unwrap();

        device.module.stop()?;

        Ok(())
    }

    pub async fn get_sensor_data(
        &self,
        data: GetSensorDataPayload,
    ) -> Result<GetSensorDataResult, ControllerError> {
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

    pub fn get_device_info_list(&self) -> Result<Vec<DeviceInfo>, ControllerError> {
        self.svc.get_device_info_list().map_err(|err| err.into())
    }

    pub fn get_device_sensor_info(
        &self,
        device_id: i32,
    ) -> Result<Vec<SensorInfo>, ControllerError> {
        let device_id = self.get_device_id(&device_id)?;

        self.svc
            .get_device_sensor_info(device_id)
            .map_err(|err| err.into())
    }

    pub async fn save_monitor_conf(
        &self,
        monitor_conf: MonitorConf,
    ) -> Result<i32, ControllerError> {
        self.svc
            .save_monitor_conf(monitor_conf)
            .await
            .map_err(|err| err.into())
    }

    pub async fn get_monitor_conf_list(
        &self,
        filter: MonitorConfListFilter,
    ) -> Result<Vec<MonitorConf>, ControllerError> {
        self.svc
            .get_monitor_conf_list(filter)
            .await
            .map_err(|err| err.into())
    }

    fn get_device(&self, id: &i32) -> Result<Arc<Mutex<Device<S, M>>>, ControllerError> {
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

impl<S: IService, M: IModule, MF: IModuleFactory<M>> Clone for Controller<S, M, MF> {
    fn clone(&self) -> Self {
        Self {
            _module_factory: std::marker::PhantomData,
            svc: self.svc.clone(),
            tokio_handle: self.tokio_handle.clone(),
            devices: self.devices.clone(),
        }
    }
}

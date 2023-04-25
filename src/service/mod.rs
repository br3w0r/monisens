mod db_model;
mod device;
mod error;
mod model;

use std::collections::HashSet;
use std::error::Error;
use tokio::io::AsyncRead;

use crate::query::integration::isqlx as sq;
use crate::tool::query_trait::{ColumnsTrait, ValuesTrait};
use crate::{repo, table, tool::validation};

pub use db_model::{
    MonitorConf, MonitorConfListFilter, MonitorLogConf, MonitorType, MonitorTypeConf, SensorData,
    SensorDataFilter, SensorDataRow, SensorDataTypeValue, Sort, SortDir,
};
pub use device::{
    DeviceID, DeviceInfo, DeviceInitState, Sensor, SensorDataEntry, SensorDataType, SensorInfo,
};
pub use error::ServiceError;
pub use model::*;

const BASE_NAME_MAX_LEN: usize = 255;

#[derive(Clone)]
pub struct Service {
    repo: repo::Repository,
    device_manager: device::DeviceManager,
}

impl Service {
    pub async fn new(repo: repo::Repository) -> Result<Self, Box<dyn Error>> {
        repo.migrate().await?;

        let device_manager = Self::init_device_manager(&repo).await?;

        Ok(Self {
            repo,
            device_manager,
        })
    }

    /// `start_device_init` starts device initialization by initializing directory
    /// for device's data, saving device's module there and saving device info
    /// in `device` table.
    ///
    /// It sets `device.init_state` to `DEVICE`
    pub async fn start_device_init<'f, F: AsyncRead + Unpin + ?Sized>(
        &self,
        name: String,
        module_file: &'f mut F,
    ) -> Result<DeviceInitData, Box<dyn Error>> {
        if let Err(err) = base_validate_name(&name) {
            return Err(Box::new(ServiceError::NameValidationErr(
                "device name".into(),
                err,
            )));
        }

        let res = self
            .device_manager
            .start_device_init(name.clone(), module_file)
            .await?;

        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name())
            .columns(db_model::Device::columns());

        db_model::Device {
            id: res.id.into(),
            name,
            module_dir: res.module_dir.clone(),
            data_dir: res.data_dir.clone(),
            init_state: db_model::DeviceInitState::Device,
        }
        .values(&mut b);

        self.repo.exec(b.insert()).await?;

        Ok(res)
    }

    /// `device_sensor_init` initializes device's sensors by creating tables in DB and
    /// binding those tables to the device by inserting into `device_sensor` table.
    ///
    /// It sets `device.init_state` to `SENSORS`
    pub async fn device_sensor_init(
        &self,
        device_id: DeviceID,
        sensors: Vec<Sensor>,
    ) -> Result<(), Box<dyn Error>> {
        // Data validation and table creation
        let mut tables = Vec::with_capacity(sensors.len());
        let mut device_sensor_query = sq::StatementBuilder::new();
        device_sensor_query
            .table(db_model::DeviceSensor::table_name())
            .columns(db_model::DeviceSensor::columns());

        let device_name = self.device_manager.get_device_name(&device_id)?;

        for (i, sensor) in sensors.iter().enumerate() {
            if let Err(err) = base_validate_name(&sensor.name) {
                return Err(Box::new(ServiceError::NameValidationErr(
                    format!("sensor[{}].name", i),
                    err,
                )));
            }

            if sensor.data_map.len() == 0 {
                return Err(Box::new(ServiceError::DeviceSensorInitErr(
                    "a sensor must specify at least one data type".into(),
                )));
            }

            let table_name = sensor_table_name(device_id, &device_name, &sensor.name);
            let mut table = table::Table::new(table_name.clone())?;

            db_model::DeviceSensor {
                device_id: device_id.get_raw(),
                sensor_table_name: table_name,
            }
            .values(&mut device_sensor_query);

            for (key, data) in sensor.data_map.iter() {
                // Validate sensor's data type name
                if key != &data.name {
                    return Err(Box::new(ServiceError::DeviceSensorInitErr(
                        "sensor's data map key is not equal to it's value.name".into(),
                    )));
                }
                if let Err(err) = base_validate_name(&data.name) {
                    return Err(Box::new(ServiceError::NameValidationErr(
                        format!("sensor[{}].data_map['{}'].name", i, &data.name),
                        err,
                    )));
                }

                // Add data type column to the table
                let mut data_type_field =
                    table::Field::new(data.name.clone(), data.typ.to_table_type())?;
                data_type_field.add_opt(table::FieldOption::NotNull)?;

                table.add_field(data_type_field)?;
            }

            tables.push(table);
        }

        // Create tables in TX
        // TODO: Retries?
        let mut tx = self.repo.tx().await?;
        for table in tables {
            tx.create_table(table).await?;
        }

        // Bind tables to device
        tx.exec(device_sensor_query.insert()).await?;

        // Update device's init_state
        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name())
            .set(
                "init_state".into(),
                db_model::DeviceInitState::Sensors.into(),
            )
            .whereq(sq::eq("id".into(), device_id.get_raw()));

        tx.exec(b.update()).await?;

        self.device_manager
            .device_sensor_init(&device_id, sensors)?;

        tx.commit().await?;

        Ok(())
    }

    /// `interrupt_device_init` interrupts device initialization if it's in `DEVICE` init_state.
    /// It deletes data in DB (in `device` table) and from the disk (in <data>/device folder)
    pub async fn interrupt_device_init(&self, id: DeviceID) -> Result<(), Box<dyn Error>> {
        let mut tx = self.repo.tx().await?;

        // Check whether device's state is 'DEVICE'
        let init_state = self.device_manager.get_device_init_state(id)?;
        if init_state != device::DeviceInitState::Device {
            return Err(Box::new(ServiceError::DeviceAlreadyInitialized(id)));
        }

        // Delete device's info
        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name())
            .whereq(sq::eq("id".into(), id.get_raw()));

        tx.exec(b.delete()).await?;

        self.device_manager.delete_device(&id).await?;
        tx.commit().await?;

        Ok(())
    }

    pub fn get_device_ids(&self) -> Vec<DeviceID> {
        self.device_manager.get_device_ids()
    }

    pub fn get_init_data_all_devices(&self) -> Vec<DeviceInitData> {
        self.device_manager.get_init_data_all_devices()
    }

    pub async fn save_sensor_data(
        &self,
        id: DeviceID,
        sensor_name: String,
        data_list: Vec<SensorData>,
    ) -> Result<(), Box<dyn Error>> {
        // TODO: data validation?
        let device_name = self.device_manager.get_device_name(&id)?;
        let table_name = quote_string(&sensor_table_name(id, &device_name, &sensor_name));
        let mut b = sq::StatementBuilder::new();

        let mut cols = Vec::with_capacity(data_list.len());
        let mut vals = Vec::with_capacity(data_list.len());

        for data in data_list {
            cols.push(data.name);
            vals.push(data.data.into())
        }

        b.table(table_name).columns(&cols).values(vals);

        self.repo.exec(b.insert()).await?;

        Ok(())
    }

    pub async fn get_sensor_data(
        &self,
        id: DeviceID,
        sensor_name: String,
        fields: Vec<String>,
        filter: db_model::SensorDataFilter,
    ) -> Result<Vec<SensorDataRow>, Box<dyn Error>> {
        let device_name = self.device_manager.get_device_name(&id)?;
        let table_name = quote_string(&sensor_table_name(id, &device_name, &sensor_name));

        let mut b = sq::StatementBuilder::new();

        b.table(table_name).columns(&fields);
        filter.apply(&mut b);

        let res = self.repo.select(b.select()).await?;

        Ok(res)
    }

    /// See `DeviceManager.get_device_info_list()` for details
    pub fn get_device_info_list(&self) -> Vec<DeviceInfo> {
        self.device_manager.get_device_info_list()
    }

    pub fn get_device_sensor_info(
        &self,
        device_id: DeviceID,
    ) -> Result<Vec<SensorInfo>, Box<dyn Error>> {
        let res = self.device_manager.get_device_sensor_info(device_id)?;

        Ok(res)
    }

    pub async fn save_monitor_conf(
        &self,
        monitor_conf: MonitorConf,
    ) -> Result<i32, Box<dyn Error>> {
        let mut b = sq::StatementBuilder::new();

        b.table(MonitorConf::table_name())
            .columns(MonitorConf::insert_columns());
        monitor_conf.values(&mut b);
        b.suffix("RETURNING id");

        let id: (i32,) = self.repo.get(b.insert()).await?;

        Ok(id.0)
    }

    pub async fn get_monitor_conf_list(
        &self,
        filter: MonitorConfListFilter,
    ) -> Result<Vec<MonitorConf>, Box<dyn Error>> {
        let mut b = sq::StatementBuilder::new();

        b.table(MonitorConf::table_name())
            .columns(MonitorConf::columns());

        filter.apply(&mut b);

        let res = self.repo.select(b.select()).await?;

        Ok(res)
    }

    async fn init_device_manager(
        repo: &repo::Repository,
    ) -> Result<device::DeviceManager, Box<dyn Error>> {
        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name()).column("*");

        let devices: Vec<db_model::Device> = repo.select(b.select()).await?;

        if devices.len() == 0 {
            return Ok(Default::default());
        }

        let mut b = sq::StatementBuilder::new();
        b.table(db_model::DeviceSensor::table_name()).column("*");

        let device_sensors: Vec<db_model::DeviceSensor> = repo.select(b.select()).await?;

        let mut sensor_table_names: HashSet<String> = HashSet::new();
        for device_sensor in device_sensors.iter() {
            sensor_table_names.insert(device_sensor.sensor_table_name.to_string());
        }

        let sensor_table_names: Vec<String> = sensor_table_names.drain().collect();

        let sensor_types: Vec<db_model::ColumnType> = {
            if sensor_table_names.len() > 0 {
                let mut b = sq::StatementBuilder::new();
                b.table("information_schema.columns".into())
                    .columns(db_model::ColumnType::columns())
                    .whereq(sq::inq("table_name".into(), sensor_table_names));

                repo.select(b.select()).await?
            } else {
                Vec::new()
            }
        };

        let device_manager = device::DeviceManager::new(&devices, &device_sensors, &sensor_types)?;

        Ok(device_manager)
    }
}

fn base_validate_name(s: &str) -> Result<(), validation::ValidationError> {
    validation::validate_len(s, BASE_NAME_MAX_LEN)?;
    validation::validate_chars(s)?;
    validation::validate_snake_case(s)
}

fn sensor_table_name(device_id: DeviceID, device_name: &str, sensor_name: &str) -> String {
    device_id.get_raw().to_string() + "-" + device_name + "__" + sensor_name
}

fn quote_string(s: &str) -> String {
    let mut res = String::with_capacity(s.len() + 2);
    res.push('"');
    res.push_str(s);
    res.push('"');

    res
}

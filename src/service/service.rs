use std::collections::HashSet;
use std::error::Error;
use std::path::Path;

use inflections::Inflect;

use super::db_model;
use super::device;
use super::error::ServiceError;

use crate::controller::{self as ctrl, interface::service::IService};
use crate::query::integration::isqlx as sq;
use crate::query::integration::isqlx::ArgType;
use crate::tool::query_trait::{ColumnsTrait, ValuesTrait};
use crate::{repo, table, tool::validation};

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
            sensor_table_names.insert(device_sensor.sensor_table_name.clone());
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

impl IService for Service {
    async fn start_device_init<'f, F: tokio::io::AsyncRead + Unpin + ?Sized>(
        &self,
        display_name: String,
        module_file: &'f mut F,
    ) -> Result<ctrl::DeviceInitData, Box<dyn std::error::Error>> {
        if let Err(err) = validation::validate_multiple_words(&display_name) {
            return Err(Box::new(ServiceError::NameValidationErr(
                "device name".into(),
                err,
            )));
        }

        let name = display_name.to_snake_case();

        let res = self
            .device_manager
            .start_device_init(name.clone(), display_name.clone(), module_file)
            .await?;

        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name())
            .columns(db_model::Device::columns());

        db_model::Device {
            id: res.id.get_raw(),
            name,
            display_name,
            module_dir: path_to_str(&res.module_dir)?,
            data_dir: path_to_str(&res.data_dir.clone())?,
            init_state: db_model::DeviceInitState::Device,
        }
        .values(&mut b);

        self.repo.exec(b.insert()).await?;

        Ok(res)
    }

    async fn device_sensor_init(
        &self,
        device_id: ctrl::DeviceID,
        sensors: Vec<ctrl::Sensor>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Data validation and table creation
        let mut tables = Vec::with_capacity(sensors.len());
        let mut device_sensor_query = sq::StatementBuilder::new();
        device_sensor_query
            .table(db_model::DeviceSensor::table_name())
            .columns(db_model::DeviceSensor::columns());

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

            let table_name = sensor_table_name(device_id.get_raw(), &sensor.name);
            let mut table = table::Table::new(table_name.clone())?;

            db_model::DeviceSensor {
                device_id: device_id.get_raw(),
                sensor_name: sensor.name.clone(),
                sensor_table_name: table_name.clone(),
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
                    table::Field::new(data.name.clone(), data_type_to_table_type(&data.typ))?;
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

    async fn interrupt_device_init(
        &self,
        id: ctrl::DeviceID,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut tx = self.repo.tx().await?;

        // Check whether device's state is 'DEVICE'
        let init_state = self.device_manager.get_device_init_state(id)?;
        if init_state != ctrl::DeviceInitState::Device {
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

    fn get_device_ids(&self) -> Vec<ctrl::DeviceID> {
        self.device_manager.get_device_ids()
    }

    fn get_init_data_all_devices(&self) -> Vec<ctrl::DeviceInitData> {
        self.device_manager.get_init_data_all_devices()
    }

    async fn save_sensor_data(
        &self,
        id: ctrl::DeviceID,
        msg: ctrl::SensorMsg,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: data validation?
        let table_name = quote_string(&sensor_table_name(id.get_raw(), &msg.name));
        let mut b = sq::StatementBuilder::new();

        let mut cols = Vec::with_capacity(msg.data.len());
        let mut vals: Vec<Box<dyn ArgType>> = Vec::with_capacity(msg.data.len());

        for data in msg.data {
            cols.push(data.name);
            vals.push(Box::<db_model::SensorDataTypeValue>::from(data.data))
        }

        b.table(table_name).columns(&cols).values(vals);

        self.repo.exec(b.insert()).await?;

        Ok(())
    }

    async fn get_sensor_data(
        &self,
        id: ctrl::DeviceID,
        sensor_name: String,
        fields: Vec<String>,
        filter: ctrl::SensorDataFilter,
    ) -> Result<Vec<ctrl::SensorDataList>, Box<dyn std::error::Error>> {
        let filter = db_model::SensorDataFilter::from(filter);

        let table_name = quote_string(&sensor_table_name(id.get_raw(), &sensor_name));

        let mut b = sq::StatementBuilder::new();

        b.table(table_name).columns(&fields);
        filter.apply(&mut b);

        let mut res: Vec<db_model::SensorDataRow> = self.repo.select(b.select()).await?;

        Ok(res
            .drain(..)
            .map(|r| ctrl::SensorDataList::from(r))
            .collect())
    }

    fn get_device_info_list(&self) -> Vec<ctrl::DeviceInfo> {
        self.device_manager.get_device_info_list()
    }

    fn get_device_sensor_info(
        &self,
        device_id: ctrl::DeviceID,
    ) -> Result<Vec<ctrl::SensorInfo>, Box<dyn std::error::Error>> {
        let res = self.device_manager.get_device_sensor_info(device_id)?;

        Ok(res)
    }

    async fn save_monitor_conf(
        &self,
        monitor_conf: ctrl::MonitorConf,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let monitor_conf = db_model::MonitorConf::from(monitor_conf);

        let mut b = sq::StatementBuilder::new();

        b.table(db_model::MonitorConf::table_name())
            .columns(db_model::MonitorConf::insert_columns());
        monitor_conf.values(&mut b);
        b.suffix("RETURNING id");

        let id: (i32,) = self.repo.get(b.insert()).await?;

        Ok(id.0)
    }

    async fn get_monitor_conf_list(
        &self,
        filter: ctrl::MonitorConfListFilter,
    ) -> Result<Vec<ctrl::MonitorConf>, Box<dyn std::error::Error>> {
        let filter = db_model::MonitorConfListFilter::from(filter);

        let mut b = sq::StatementBuilder::new();

        b.table(db_model::MonitorConf::table_name())
            .columns(db_model::MonitorConf::columns());

        filter.apply(&mut b);

        let mut res: Vec<db_model::MonitorConf> = self.repo.select(b.select()).await?;

        Ok(res.drain(..).map(|v| ctrl::MonitorConf::from(v)).collect())
    }
}

fn path_to_str<P: AsRef<Path>>(path: P) -> Result<String, ServiceError> {
    let p = path.as_ref().to_str().ok_or(ServiceError::InvalidPath)?;

    Ok(p.to_string())
}

fn base_validate_name(s: &str) -> Result<(), validation::ValidationError> {
    validation::validate_len(s, BASE_NAME_MAX_LEN)?;
    validation::validate_chars(s)?;
    validation::validate_snake_case(s)
}

fn sensor_table_name(device_id: i32, sensor_name: &str) -> String {
    device_id.to_string() + "__" + sensor_name
}

pub fn data_type_to_table_type(val: &ctrl::SensorDataType) -> table::FieldType {
    match val {
        ctrl::SensorDataType::Int16 => table::FieldType::Int16,
        ctrl::SensorDataType::Int32 => table::FieldType::Int32,
        ctrl::SensorDataType::Int64 => table::FieldType::Int64,
        ctrl::SensorDataType::Float32 => table::FieldType::Float32,
        ctrl::SensorDataType::Float64 => table::FieldType::Float64,
        ctrl::SensorDataType::Timestamp => table::FieldType::Timestamp,
        ctrl::SensorDataType::String => table::FieldType::Text,
        ctrl::SensorDataType::JSON => table::FieldType::JSON,
    }
}

fn quote_string(s: &str) -> String {
    let mut res = String::with_capacity(s.len() + 2);
    res.push('"');
    res.push_str(s);
    res.push('"');

    res
}

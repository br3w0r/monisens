use std::collections::HashSet;
use std::error::Error;
use std::path::Path;

use inflections::Inflect;

use super::db_model;
use super::device;
use super::error::ServiceError as InternalServiceError;

use crate::controller::{
    self as ctrl,
    error::{CommonError, ErrorType},
    interface::service::IService,
};
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
    ) -> Result<ctrl::DeviceInitData, CommonError> {
        if let Err(err) = validation::validate_multiple_words(&display_name) {
            return Err(CommonError::new(
                ErrorType::InvalidInput,
                "failed to validate display_name",
            )
            .with_source(err));
        }

        let name = display_name.to_snake_case();

        let res = self
            .device_manager
            .start_device_init(name.clone(), display_name.clone(), module_file)
            .await
            .map_err(|err| {
                CommonError::new(ErrorType::Internal, "failed to start device init")
                    .with_source(err)
            })?;

        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name())
            .columns(db_model::Device::columns());

        let module_dir = path_to_str(&res.module_dir).map_err(|err| {
            CommonError::new(
                ErrorType::Internal,
                "failed to convert module path to string",
            )
            .with_source(err)
        })?;

        let data_dir = path_to_str(&res.data_dir.clone()).map_err(|err| {
            CommonError::new(ErrorType::Internal, "failed to convert data path to string")
                .with_source(err)
        })?;

        db_model::Device {
            id: res.id.get_raw(),
            name,
            display_name,
            module_dir,
            data_dir,
            init_state: db_model::DeviceInitState::Device,
        }
        .values(&mut b);

        self.repo
            .exec(b.insert())
            .await
            .map_err(|err| err.to_common_err("failed to save device info"))?;

        Ok(res)
    }

    async fn device_sensor_init(
        &self,
        device_id: ctrl::DeviceID,
        sensors: Vec<ctrl::Sensor>,
    ) -> Result<(), CommonError> {
        // Data validation and table creation
        let mut tables = Vec::with_capacity(sensors.len());
        let mut device_sensor_query = sq::StatementBuilder::new();
        device_sensor_query
            .table(db_model::DeviceSensor::table_name())
            .columns(db_model::DeviceSensor::columns());

        for (i, sensor) in sensors.iter().enumerate() {
            if let Err(err) = base_validate_name(&sensor.name) {
                return Err(CommonError::new(
                    ErrorType::Internal,
                    format!("invalid sensor name for sensor[{}]", i),
                )
                .with_source(err));
            }

            if sensor.data_map.len() == 0 {
                return Err(CommonError::new(
                    ErrorType::Internal,
                    "a sensor must specify at least one data type",
                ));
            }

            let table_name = sensor_table_name(device_id.get_raw(), &sensor.name);
            let mut table = table::Table::new(table_name.clone()).map_err(|err| {
                CommonError::new(ErrorType::Internal, "failed to create table structure")
                    .with_source(err)
            })?;

            db_model::DeviceSensor {
                device_id: device_id.get_raw(),
                sensor_name: sensor.name.clone(),
                sensor_table_name: table_name.clone(),
            }
            .values(&mut device_sensor_query);

            for (key, data) in sensor.data_map.iter() {
                // Validate sensor's data type name
                if key != &data.name {
                    return Err(CommonError::new(
                        ErrorType::Internal,
                        "sensor's data map key is not equal to it's value.name",
                    ));
                }
                if let Err(err) = base_validate_name(&data.name) {
                    return Err(CommonError::new(
                        ErrorType::Internal,
                        format!(
                            "invalid data name for sensor[{}] and data name '{}'",
                            i, &data.name
                        ),
                    )
                    .with_source(err));
                }

                // Add data type column to the table
                let mut data_type_field =
                    table::Field::new(data.name.clone(), data_type_to_table_type(&data.typ))
                        .map_err(|err| {
                            CommonError::new(ErrorType::Internal, "failed to create field")
                                .with_source(err)
                        })?;
                data_type_field
                    .add_opt(table::FieldOption::NotNull)
                    .map_err(|err| {
                        CommonError::new(ErrorType::Internal, "failed to add field option to field")
                            .with_source(err)
                    })?;

                table.add_field(data_type_field).map_err(|err| {
                    CommonError::new(ErrorType::Internal, "failed to add field to table")
                        .with_source(err)
                })?;
            }

            tables.push(table);
        }

        // Create tables in TX
        // TODO: Retries?
        let mut tx = self
            .repo
            .tx()
            .await
            .map_err(|err| err.to_common_err("failed to start transaction"))?;
        for table in tables {
            tx.create_table(table).await.map_err(|err| {
                CommonError::new(ErrorType::Internal, "failed to create table in DB")
                    .with_source(err)
            })?;
        }

        // Bind tables to device
        tx.exec(device_sensor_query.insert()).await.map_err(|err| {
            CommonError::new(
                ErrorType::Internal,
                "failed to bind sensors to device in DB",
            )
            .with_source(err)
        })?;

        // Update device's init_state
        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name())
            .set(
                "init_state".into(),
                db_model::DeviceInitState::Sensors.into(),
            )
            .whereq(sq::eq("id".into(), device_id.get_raw()));

        tx.exec(b.update()).await.map_err(|err| {
            CommonError::new(ErrorType::Internal, "failed to update device's init state")
                .with_source(err)
        })?;

        self.device_manager
            .device_sensor_init(&device_id, sensors)
            .map_err(|err| {
                CommonError::new(
                    ErrorType::Internal,
                    "failed to init device's sensors in device manager",
                )
                .with_source(err)
            })?;

        tx.commit().await.map_err(|err| {
            CommonError::new(ErrorType::Internal, "failed to commit transaction").with_source(err)
        })?;

        Ok(())
    }

    async fn interrupt_device_init(&self, id: ctrl::DeviceID) -> Result<(), CommonError> {
        let mut tx = self
            .repo
            .tx()
            .await
            .map_err(|err| err.to_common_err("failed to start transaction"))?;

        // Check whether device's state is 'DEVICE'
        let init_state = self
            .device_manager
            .get_device_init_state(id)
            .map_err(|err| {
                CommonError::new(ErrorType::Internal, "failed to get device's init state")
                    .with_source(err)
            })?;
        if init_state != ctrl::DeviceInitState::Device {
            return Err(CommonError::new(
                ErrorType::FailedPrecondition,
                "device's init state is not 'Device'",
            ));
        }

        // Delete device's info
        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name())
            .whereq(sq::eq("id".into(), id.get_raw()));

        tx.exec(b.delete()).await.map_err(|err| {
            CommonError::new(ErrorType::Internal, "failed to delete device info from DB")
                .with_source(err)
        })?;

        self.device_manager
            .delete_device(&id)
            .await
            .map_err(|err| {
                CommonError::new(
                    ErrorType::Internal,
                    "failed to delete device from device manager",
                )
                .with_source(err)
            })?;
        tx.commit().await.map_err(|err| {
            CommonError::new(ErrorType::Internal, "failed to commit transaction").with_source(err)
        })?;

        Ok(())
    }

    fn get_device_ids(&self) -> Result<Vec<ctrl::DeviceID>, CommonError> {
        Ok(self.device_manager.get_device_ids())
    }

    fn get_init_data_all_devices(&self) -> Result<Vec<ctrl::DeviceInitData>, CommonError> {
        Ok(self.device_manager.get_init_data_all_devices())
    }

    async fn save_sensor_data(
        &self,
        id: ctrl::DeviceID,
        msg: ctrl::SensorMsg,
    ) -> Result<(), CommonError> {
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

        self.repo
            .exec(b.insert())
            .await
            .map_err(|err| err.to_common_err("failed to save sensor data"))?;

        Ok(())
    }

    async fn get_sensor_data(
        &self,
        id: ctrl::DeviceID,
        sensor_name: String,
        fields: Vec<String>,
        filter: ctrl::SensorDataFilter,
    ) -> Result<Vec<ctrl::SensorDataList>, CommonError> {
        let filter = db_model::SensorDataFilter::from(filter);

        let table_name = quote_string(&sensor_table_name(id.get_raw(), &sensor_name));

        let mut b = sq::StatementBuilder::new();

        b.table(table_name).columns(&fields);
        filter.apply(&mut b);

        let mut res: Vec<db_model::SensorDataRow> = self
            .repo
            .select(b.select())
            .await
            .map_err(|err| err.to_common_err("failed to get sensor data"))?;

        Ok(res
            .drain(..)
            .map(|r| ctrl::SensorDataList::from(r))
            .collect())
    }

    fn get_device_info_list(&self) -> Result<Vec<ctrl::DeviceInfo>, CommonError> {
        Ok(self.device_manager.get_device_info_list())
    }

    fn get_device_sensor_info(
        &self,
        device_id: ctrl::DeviceID,
    ) -> Result<Vec<ctrl::SensorInfo>, CommonError> {
        let res = self
            .device_manager
            .get_device_sensor_info(device_id)
            .map_err(|err| {
                CommonError::new(ErrorType::Internal, "failed to get device sensor info")
                    .with_source(err)
            })?;

        Ok(res)
    }

    async fn save_monitor_conf(&self, monitor_conf: ctrl::MonitorConf) -> Result<i32, CommonError> {
        let monitor_conf = db_model::MonitorConf::from(monitor_conf);

        let mut b = sq::StatementBuilder::new();

        b.table(db_model::MonitorConf::table_name())
            .columns(db_model::MonitorConf::insert_columns());
        monitor_conf.values(&mut b);
        b.suffix("RETURNING id");

        let id: (i32,) = self
            .repo
            .get(b.insert())
            .await
            .map_err(|err| err.to_common_err("failed to save monitor conf"))?;

        Ok(id.0)
    }

    async fn get_monitor_conf_list(
        &self,
        filter: ctrl::MonitorConfListFilter,
    ) -> Result<Vec<ctrl::MonitorConf>, CommonError> {
        let filter = db_model::MonitorConfListFilter::from(filter);

        let mut b = sq::StatementBuilder::new();

        b.table(db_model::MonitorConf::table_name())
            .columns(db_model::MonitorConf::columns());

        filter.apply(&mut b);

        let mut res: Vec<db_model::MonitorConf> = self
            .repo
            .select(b.select())
            .await
            .map_err(|err| err.to_common_err("failed to get monitor conf list"))?;

        Ok(res.drain(..).map(|v| ctrl::MonitorConf::from(v)).collect())
    }
}

fn path_to_str<P: AsRef<Path>>(path: P) -> Result<String, InternalServiceError> {
    let p = path
        .as_ref()
        .to_str()
        .ok_or(InternalServiceError::InvalidPath)?;

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

mod db_model;
mod device;
mod model;

use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use std::error::Error;
use tokio::io::AsyncRead;

use crate::query::integration::isqlx as sq;
use crate::tool::query_trait::{ColumnsTrait, InsertTrait};
use crate::{repo, tool::validation};

pub use device::DeviceID;
pub use model::*;

const DEVICE_NAME_MAX_LEN: usize = 255;

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

    pub async fn start_device_init<'f, F: AsyncRead + Unpin + ?Sized>(
        &self,
        name: String,
        module_file: &'f mut F,
    ) -> Result<DeviceInitData, Box<dyn Error>> {
        validate_device_name(&name)?;

        let (id, module_dir, data_dir) = self
            .device_manager
            .start_device_init(name.clone(), module_file)
            .await?;

        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name())
            .columns(db_model::Device::columns());

        db_model::Device {
            id: id.into(),
            name,
            module_dir: module_dir.clone(),
            data_dir: data_dir.clone(),
            init_state: db_model::DeviceInitState::Device,
        }
        .insert(&mut b);

        self.repo.exec(b.insert()).await?;

        Ok(DeviceInitData {
            id,
            module_dir,
            data_dir,
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

fn validate_device_name(s: &str) -> Result<(), validation::ValidationError> {
    validation::validate_len(s, DEVICE_NAME_MAX_LEN);
    validation::validate_chars(s)?;
    validation::validate_snake_case(s)
}

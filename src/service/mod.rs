mod db_model;
mod device;

use std::collections::HashSet;
use std::error::Error;

use crate::query::integration::isqlx as sq;
use crate::{repo, tool::validation};

pub use device::DeviceID;

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

    pub fn device_count(&self) -> usize {
        self.device_manager.device_count()
    }

    async fn init_device_manager(
        repo: &repo::Repository,
    ) -> Result<device::DeviceManager, Box<dyn Error>> {
        let mut b = sq::StatementBuilder::new();
        b.table(db_model::Device::table_name()).column("*".into());

        let devices: Vec<db_model::Device> = repo.select(b.select()).await?;

        if devices.len() == 0 {
            return Ok(Default::default());
        }

        let mut b = sq::StatementBuilder::new();
        b.table(db_model::DeviceSensor::table_name())
            .column("*".into());

        let device_sensors: Vec<db_model::DeviceSensor> = repo.select(b.select()).await?;

        let mut sensor_table_names: HashSet<String> = HashSet::new();
        for device_sensor in device_sensors.iter() {
            sensor_table_names.insert(device_sensor.sensor_table_name.to_string());
        }

        let sensor_table_names: Vec<String> = sensor_table_names.drain().collect();

        let mut b = sq::StatementBuilder::new();
        b.table("information_schema.columns".into())
            .columns(&[
                "table_name".into(),
                "column_name".into(),
                "udt_name".into(),
            ])
            .whereq(sq::inq("table_name".into(), sensor_table_names));

        let sensor_types: Vec<db_model::ColumnType> = repo.select(b.select()).await?;

        let device_manager = device::DeviceManager::new(&devices, &device_sensors, &sensor_types)?;

        Ok(device_manager)
    }
}

fn validate_device_name(s: &str) -> Result<(), validation::ValidationError> {
    validation::validate_len(s, DEVICE_NAME_MAX_LEN);
    validation::validate_chars(s)?;
    validation::validate_snake_case(s)
}

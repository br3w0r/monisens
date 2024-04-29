mod module;
mod service;

pub mod internal;

use std::collections::HashMap;

pub use module::*;
pub use service::*;

pub struct DeviceConnData {
    pub id: DeviceID,
    pub conn_params: Vec<ConnParamConf>,
}

pub type GetSensorDataResult = Vec<HashMap<String, SensorData>>;

pub struct GetSensorDataPayload {
    pub device_id: i32,
    pub sensor: String,
    pub fields: Vec<String>,
    pub sort: Sort,
    pub from: Option<module::SensorDataTypeValue>,
    pub limit: Option<i32>,
}

impl GetSensorDataPayload {
    pub fn to_sensor_data_filter(&self) -> service::SensorDataFilter {
        let mut res = service::SensorDataFilter::default();

        if let Some(ref from) = self.from {
            if self.sort.order == SortDir::ASC {
                res.to = Some((self.sort.field.clone(), from.clone()));
            } else {
                res.from = Some((self.sort.field.clone(), from.clone()));
            }
        }

        res.limit = self.limit.clone();
        res.sort = Some(self.sort.clone().into());

        res
    }
}

pub fn sensor_data_result_from_service(mut value: Vec<SensorDataList>) -> GetSensorDataResult {
    value
        .drain(..)
        .map(|mut v| v.drain(..).map(|v| (v.name.clone(), v)).collect())
        .collect()
}

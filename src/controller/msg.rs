use std::sync::{Arc, Mutex};

use tokio::{runtime::Handle, task};

use crate::{
    module,
    service::{self, DeviceID},
};

#[derive(Clone)]
pub struct Handler(Arc<Mutex<HandlerImpl>>);

impl Handler {
    pub fn new(device_id: DeviceID, svc: service::Service, tokio_handle: Handle) -> Self {
        Handler(Arc::new(Mutex::new(HandlerImpl::new(
            device_id,
            svc,
            tokio_handle,
        ))))
    }
}

impl module::MsgHandler for Handler {
    fn handle_msg(&self, msg: module::Message) {
        let mut h = self.0.lock().unwrap();
        h.handle_msg(msg);
    }
}

struct HandlerImpl {
    device_id: DeviceID,
    svc: service::Service,
    tokio_handle: Handle,
}

impl HandlerImpl {
    fn new(device_id: DeviceID, svc: service::Service, tokio_handle: Handle) -> Self {
        Self {
            device_id,
            svc,
            tokio_handle,
        }
    }

    fn handle_msg(&mut self, msg: module::Message) {
        if let module::MessageType::Sensor(mut msg) = msg.msg {
            // TODO: error handling
            task::block_in_place(move || {
                self.tokio_handle.block_on(
                    self.svc.save_sensor_data(
                        self.device_id,
                        msg.name,
                        msg.data
                            .drain(..)
                            .map(|v| service_sensor_msg_data_from_module(v))
                            .collect(),
                    ),
                )
            })
            .unwrap();
        }
    }
}

fn service_sensor_msg_data_from_module(value: module::SensorMsgData) -> service::SensorData {
    service::SensorData {
        name: value.name,
        data: match value.data {
            module::SensorMsgDataType::Int16(v) => service::SensorDataTypeValue::Int16(v),
            module::SensorMsgDataType::Int32(v) => service::SensorDataTypeValue::Int32(v),
            module::SensorMsgDataType::Int64(v) => service::SensorDataTypeValue::Int64(v),
            module::SensorMsgDataType::Float32(v) => service::SensorDataTypeValue::Float32(v),
            module::SensorMsgDataType::Float64(v) => service::SensorDataTypeValue::Float64(v),
            module::SensorMsgDataType::Timestamp(v) => service::SensorDataTypeValue::Timestamp(
                chrono::NaiveDateTime::from_timestamp_opt(v.clone(), 0).unwrap(),
            ),
            module::SensorMsgDataType::String(v) => service::SensorDataTypeValue::String(v),
            module::SensorMsgDataType::JSON(v) => service::SensorDataTypeValue::JSON(v),
        },
    }
}

use std::sync::{Arc, Mutex};

use tokio::{runtime::Handle, task};

use crate::module;
use super::interface::service;
use super::model;

#[derive(Clone)]
pub struct Handler<S: service::IService>(Arc<Mutex<HandlerImpl<S>>>);

impl<S: service::IService> Handler<S> {
    pub fn new(device_id: model::DeviceID, svc: S, tokio_handle: Handle) -> Self {
        Handler(Arc::new(Mutex::new(HandlerImpl::new(
            device_id,
            svc,
            tokio_handle,
        ))))
    }
}

impl<S: service::IService> module::MsgHandler for Handler<S> {
    fn handle_msg(&self, msg: module::Message) {
        let mut h = self.0.lock().unwrap();
        h.handle_msg(msg);
    }
}

struct HandlerImpl<S: service::IService> {
    device_id: model::DeviceID,
    svc: S,
    tokio_handle: Handle,
}

impl<S: service::IService> HandlerImpl<S> {
    fn new(device_id: model::DeviceID, svc: S, tokio_handle: Handle) -> Self {
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

fn service_sensor_msg_data_from_module(value: module::SensorMsgData) -> model::SensorData {
    model::SensorData {
        name: value.name,
        data: match value.data {
            module::SensorMsgDataType::Int16(v) => model::SensorDataTypeValue::Int16(v),
            module::SensorMsgDataType::Int32(v) => model::SensorDataTypeValue::Int32(v),
            module::SensorMsgDataType::Int64(v) => model::SensorDataTypeValue::Int64(v),
            module::SensorMsgDataType::Float32(v) => model::SensorDataTypeValue::Float32(v),
            module::SensorMsgDataType::Float64(v) => model::SensorDataTypeValue::Float64(v),
            module::SensorMsgDataType::Timestamp(v) => model::SensorDataTypeValue::Timestamp(
                chrono::NaiveDateTime::from_timestamp_opt(v.clone(), 0).unwrap(),
            ),
            module::SensorMsgDataType::String(v) => model::SensorDataTypeValue::String(v),
            module::SensorMsgDataType::JSON(v) => model::SensorDataTypeValue::JSON(v),
        },
    }
}

use std::sync::{Arc, Mutex};

use tokio::{runtime::Handle, task};

use super::interface::{module, service};
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
    fn handle_msg(&self, msg: model::Message) {
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

    fn handle_msg(&mut self, msg: model::Message) {
        if let model::MessageType::Sensor(mut msg) = msg.msg {
            // TODO: error handling
            task::block_in_place(move || {
                self.tokio_handle
                    .block_on(self.svc.save_sensor_data(self.device_id, msg))
            })
            .unwrap();
        }
    }
}

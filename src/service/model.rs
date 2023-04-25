use super::device;

#[derive(Debug)]
pub struct DeviceInitData {
    pub id: device::DeviceID,
    pub module_dir: String,
    pub module_file: String,
    pub data_dir: String,
    pub init_state: device::DeviceInitState,
}

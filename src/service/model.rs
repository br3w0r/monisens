use std::path::PathBuf;

use super::device;

#[derive(Debug)]
pub struct DeviceInitData {
    pub id: device::DeviceID,
    pub module_dir: PathBuf,
    pub module_file: PathBuf,
    pub data_dir: PathBuf,
    pub full_data_dir: PathBuf,
    pub init_state: device::DeviceInitState,
}

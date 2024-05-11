use super::bindings_gen as bg;
use super::conv;
use super::error::{ComError, ModuleError};

use libc::c_void;
use std::collections::HashMap;

use crate::controller;
use crate::controller::interface::module::MsgHandler;

pub const VERSION: u8 = 1;

pub struct Handle(*const c_void);

impl Handle {
    pub fn new() -> Self {
        Handle(std::ptr::null())
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn handler_ptr(&mut self) -> *mut *mut c_void {
        self as *mut Self as *mut *mut c_void
    }

    pub fn handler(&mut self) -> *mut c_void {
        self.0 as _
    }
}

unsafe impl Send for Handle {}

pub extern "C" fn device_conn_info_callback(obj: *mut c_void, info: *mut bg::ConfInfo) {
    conf_info(obj as _, info);
}

fn build_conf_info(info: *mut bg::ConfInfo) -> Result<controller::ConfInfo, ModuleError> {
    if unsafe { (*info).confs }.is_null() {
        return Err(ModuleError::InvalidPointer("conf_info.confs"));
    }

    let confs = unsafe { std::slice::from_raw_parts((*info).confs, (*info).confs_len as _) };
    let mut res = controller::ConfInfo::with_capacity(unsafe { (*info).confs_len } as _);

    for conf in confs {
        let data = build_conf_info_entry_data(conf)?;

        res.push(controller::ConfInfoEntry {
            id: conf.id,
            name: conv::str_from_c_char(conf.name),
            data: data,
        });
    }

    Ok(res)
}

fn build_conf_info_entry_data(
    conf: &bg::ConfInfoEntry,
) -> Result<controller::ConfInfoEntryType, ModuleError> {
    match conf.typ {
        bg::ConfInfoEntryType::ConfInfoEntryTypeSection => {
            let section = build_conf_info(conf.data as *mut bg::ConfInfo)?;

            Ok(controller::ConfInfoEntryType::Section(section))
        }
        bg::ConfInfoEntryType::ConfInfoEntryTypeString => {
            let data = unsafe { *(conf.data as *mut bg::ConfInfoEntryString) };

            Ok(controller::ConfInfoEntryType::String(
                controller::ConfInfoEntryString {
                    required: data.required,
                    default: conv::option_str_from_c_char(data.def),
                    min_len: nullable_into_option(data.min_len),
                    max_len: nullable_into_option(data.max_len),
                    match_regex: conv::option_str_from_c_char(data.match_regex),
                },
            ))
        }
        bg::ConfInfoEntryType::ConfInfoEntryTypeInt => {
            let data = unsafe { *(conf.data as *mut bg::ConfInfoEntryInt) };

            Ok(controller::ConfInfoEntryType::Int(
                controller::ConfInfoEntryInt {
                    required: data.required,
                    default: nullable_into_option(data.def),
                    lt: nullable_into_option(data.lt),
                    gt: nullable_into_option(data.gt),
                    neq: nullable_into_option(data.neq),
                },
            ))
        }
        bg::ConfInfoEntryType::ConfInfoEntryTypeIntRange => {
            let data = unsafe { *(conf.data as *mut bg::ConfInfoEntryIntRange) };

            Ok(controller::ConfInfoEntryType::IntRange(
                controller::ConfInfoEntryIntRange {
                    required: data.required,
                    def_from: nullable_into_option(data.def_from),
                    def_to: nullable_into_option(data.def_to),
                    min: data.min,
                    max: data.max,
                },
            ))
        }
        bg::ConfInfoEntryType::ConfInfoEntryTypeFloat => {
            let data = unsafe { *(conf.data as *mut bg::ConfInfoEntryFloat) };

            Ok(controller::ConfInfoEntryType::Float(
                controller::ConfInfoEntryFloat {
                    required: data.required,
                    default: nullable_into_option(data.def),
                    lt: nullable_into_option(data.lt),
                    gt: nullable_into_option(data.gt),
                    neq: nullable_into_option(data.neq),
                },
            ))
        }
        bg::ConfInfoEntryType::ConfInfoEntryTypeFloatRange => {
            let data = unsafe { *(conf.data as *mut bg::ConfInfoEntryFloatRange) };

            Ok(controller::ConfInfoEntryType::FloatRange(
                controller::ConfInfoEntryFloatRange {
                    required: data.required,
                    def_from: nullable_into_option(data.def_from),
                    def_to: nullable_into_option(data.def_to),
                    min: data.min,
                    max: data.max,
                },
            ))
        }
        bg::ConfInfoEntryType::ConfInfoEntryTypeJSON => {
            let data = unsafe { *(conf.data as *mut bg::ConfInfoEntryJSON) };

            Ok(controller::ConfInfoEntryType::JSON(
                controller::ConfInfoEntryJSON {
                    required: data.required,
                    default: conv::option_str_from_c_char(data.def),
                },
            ))
        }
        bg::ConfInfoEntryType::ConfInfoEntryTypeChoiceList => {
            let data = unsafe { *(conf.data as *mut bg::ConfInfoEntryChoiceList) };

            let mut entry = controller::ConfInfoEntryChoiceList {
                required: data.required,
                default: nullable_into_option(data.def),
                choices: Vec::with_capacity(data.chioces_len as _),
            };

            for choice in unsafe { std::slice::from_raw_parts(data.choices, data.chioces_len as _) }
            {
                entry.choices.push(conv::str_from_c_char(*choice));
            }

            Ok(controller::ConfInfoEntryType::ChoiceList(entry))
        }
    }
}

pub type ConfInfoRec = Result<controller::ConfInfo, ModuleError>;

fn conf_info(res: *mut ConfInfoRec, info: *mut bg::ConfInfo) {
    if info.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("conf"));
        }
        return;
    }

    unsafe {
        *res = build_conf_info(info);
    }
}

pub extern "C" fn device_conf_info_callback(obj: *mut c_void, info: *mut bg::ConfInfo) {
    conf_info(obj as _, info);
}

pub fn build_conf(confs: &Vec<bg::ConfEntry>) -> bg::Conf {
    bg::Conf {
        confs: confs.as_ptr() as _,
        confs_len: confs.len() as _,
    }
}

pub fn bg_sensor_type_infos_to_sensor_vec(
    infos: *mut bg::SensorTypeInfos,
) -> Result<Vec<controller::Sensor>, ModuleError> {
    let infos_slice = unsafe {
        std::slice::from_raw_parts(
            (*infos).sensor_type_infos,
            (*infos).sensor_type_infos_len as _,
        )
    };

    let mut res_infos = Vec::with_capacity(infos_slice.len());
    for info in infos_slice {
        let data_type_infos_slice = unsafe {
            std::slice::from_raw_parts(info.data_type_infos, info.data_type_infos_len as _)
        };

        let mut res_data_type_infos_map = HashMap::with_capacity(data_type_infos_slice.len());
        for data_type_info in data_type_infos_slice {
            let name = conv::str_from_c_char(data_type_info.name);
            res_data_type_infos_map.insert(
                name.clone(),
                controller::SensorDataEntry {
                    name,
                    typ: conv::bg_sensor_data_type_to_ctrl(&data_type_info.typ),
                },
            );
        }

        res_infos.push(controller::Sensor {
            name: conv::str_from_c_char(info.name),
            data_map: res_data_type_infos_map,
        })
    }

    Ok(res_infos)
}

pub type SensorTypeInfosRec = Result<Vec<controller::Sensor>, ModuleError>;

fn sensor_type_infos(res: *mut SensorTypeInfosRec, infos: *mut bg::SensorTypeInfos) {
    if infos.is_null() {
        unsafe {
            *res = Err(ModuleError::InvalidPointer("sensor_type_infos"));
        }
        return;
    }

    unsafe {
        *res = bg_sensor_type_infos_to_sensor_vec(infos);
    }
}

pub extern "C" fn sensor_type_infos_callback(obj: *mut c_void, infos: *mut bg::SensorTypeInfos) {
    sensor_type_infos(obj as _, infos);
}

pub struct MsgHandle(Box<dyn MsgHandler>);

impl MsgHandle {
    pub fn new<H: MsgHandler + 'static>(msg_handler: H) -> Self {
        Self(Box::new(msg_handler))
    }
}

pub extern "C" fn handle_msg_callback(handler: *mut c_void, msg_data: bg::Message) {
    let h = handler as *const MsgHandle;

    unsafe {
        let data = conv::bg_message_to_ctrl(&msg_data);
        let h = &(*h).0;
        h.handle_msg(data);
    }
}

// ------------------- Utility functions -------------------

pub fn convert_com_error(err: u8) -> Result<(), ComError> {
    match err {
        0 => Ok(()),
        1 => Err(ComError::ConnectionError),
        2 => Err(ComError::InvalidArgument),
        _ => Err(ComError::Unknown),
    }
}

fn nullable_into_option<T: Copy>(nullable_val: *mut T) -> Option<T> {
    if nullable_val.is_null() {
        None
    } else {
        Some(unsafe { *nullable_val })
    }
}

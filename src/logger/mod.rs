use core::fmt;
use lazy_static::lazy_static;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime},
    vec,
};

lazy_static! {
    static ref LOGGER: Logger = Logger::new();
}

pub fn log_kv(level: LogLevel, msg: &str, kvs: Option<Vec<KV>>) {
    LOGGER.log(level, msg, KV::merge(KV::default_kvs(), kvs));
}

pub fn info_kv(msg: &str, kvs: Option<Vec<KV>>) {
    LOGGER.log(LogLevel::Info, msg, KV::merge(KV::default_kvs(), kvs));
}

pub fn warn_kv(msg: &str, kvs: Option<Vec<KV>>) {
    LOGGER.log(LogLevel::Warn, msg, KV::merge(KV::default_kvs(), kvs));
}

pub fn error_kv<'log>(msg: &'log str, kvs: Option<Vec<KV<'log>>>) {
    LOGGER.log(LogLevel::Error, msg, KV::merge(KV::default_kvs(), kvs));
}

pub fn register_writer(writer: LogWriterType) {
    LOGGER.clone().register_writer(writer);
}

#[derive(Clone, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Fatal,
}

pub trait KVValue {
    fn write_value(&self, f: &mut Box<dyn std::io::Write + Send>) -> std::io::Result<()>;
}

pub enum KVType<'kv> {
    Timestamp(Duration),
    KVValue(Box<dyn KVValue + 'kv>),
    Any(Box<dyn std::fmt::Debug + 'kv>),
}

pub struct KV<'kv> {
    key: String,
    value: KVType<'kv>,
}

impl<'kv> KV<'kv> {
    pub fn new(key: String, value: KVType<'kv>) -> Self {
        Self { key, value }
    }

    fn default_kvs() -> Vec<KV<'kv>> {
        vec![KV {
            key: "timestamp".into(),
            value: KVType::Timestamp(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_else(|err| err.duration()),
            ),
        }]
    }

    fn merge(to: Vec<KV<'kv>>, from: Option<Vec<KV<'kv>>>) -> Vec<KV<'kv>> {
        match from {
            Some(kvs) => {
                let mut v = Vec::with_capacity(to.len() + kvs.len());

                for kv in to {
                    v.push(kv);
                }

                for kv in kvs {
                    v.push(kv);
                }

                v
            }
            None => to,
        }
    }
}

pub trait LogWriter {
    fn log<'log>(&mut self, level: &'log LogLevel, msg: &'log str, kvs: &Vec<KV<'log>>);
}

pub type LogWriterType = Arc<Mutex<dyn LogWriter + Send>>;

#[derive(Clone)]
struct Logger {
    log_writers: Arc<RwLock<Vec<LogWriterType>>>,
}

impl Logger {
    fn new() -> Self {
        Self {
            log_writers: Arc::new(RwLock::new(vec![StdLogger::new()])),
        }
    }

    fn register_writer(&mut self, writer: LogWriterType) {
        self.log_writers.write().unwrap().push(writer);
    }

    fn log(&self, level: LogLevel, msg: &str, kvs: Vec<KV>) {
        for w in self.log_writers.read().unwrap().iter() {
            w.lock().unwrap().log(&level, &msg, &kvs);
        }
    }
}

struct StdLogger {
    std_out_writer: Box<dyn std::io::Write + Send>,
}

impl StdLogger {
    fn new() -> LogWriterType {
        Arc::new(Mutex::new(Self {
            std_out_writer: Box::new(std::io::stdout()),
        }))
    }
}

impl LogWriter for StdLogger {
    fn log<'log>(&mut self, level: &'log LogLevel, msg: &'log str, kvs: &Vec<KV<'log>>) {
        print!(">>> level: {:?}; msg: {}, KVs: {{", level, msg);
        for (i, kv) in kvs.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }

            print!("[{}]: ", kv.key);
            match kv.value {
                KVType::Timestamp(ref d) => print!("{:?}, ", d),
                KVType::KVValue(ref val) => {
                    let _ = val.write_value(&mut self.std_out_writer);
                }
                KVType::Any(ref val) => print!("{:?}, ", val),
            }
        }
        print!("}}\n");
    }
}

#[macro_export]
macro_rules! kvs {
    ($($key:expr => $value:expr),+) => (
        Some(vec![$($crate::logger::KV::new($key.into(), $value),)+])
    )
}

#[macro_export]
macro_rules! kv_any {
    ($value:expr) => {
        $crate::logger::KVType::Any(Box::new($value))
    };
}

#[macro_export]
macro_rules! kv_val {
    ($value:expr) => {
        $crate::logger::KVType::KVValue(Box::new($value))
    };
}

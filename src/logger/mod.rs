use lazy_static::lazy_static;
use std::{
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime},
    vec, fmt::{Debug, Write},
};

lazy_static! {
    static ref LOGGER: Logger = Logger::new();
}

pub fn log_kv(level: LogLevel, msg: String, kvs: Option<Vec<KV>>) {
    LOGGER.log(level, msg, KV::merge(KV::default_kvs(), kvs));
}

pub fn info_kv(msg: String, kvs: Option<Vec<KV>>) {
    LOGGER.log(LogLevel::Info, msg, KV::merge(KV::default_kvs(), kvs));
}

pub fn warn_kv(msg: String, kvs: Option<Vec<KV>>) {
    LOGGER.log(LogLevel::Warn, msg, KV::merge(KV::default_kvs(), kvs));
}

pub fn error_kv(msg: String, kvs: Option<Vec<KV>>) {
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

#[derive(Debug, Clone)]
enum KVType {
    Timestamp(Duration),
    Any(Arc<dyn std::fmt::Debug>),
}

#[derive(Clone)]
pub struct KV {
    key: String,
    value: KVType,
}

impl KV {
    pub fn new<T: std::fmt::Debug + 'static>(key: String, value: T) -> Self {
        Self {
            key,
            value: KVType::Any(Arc::new(value)),
        }
    }

    fn default_kvs() -> Vec<KV> {
        vec![KV {
            key: "timestamp".into(),
            value: KVType::Timestamp(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_else(|err| err.duration()),
            ),
        }]
    }

    fn merge(to: Vec<KV>, from: Option<Vec<KV>>) -> Vec<KV> {
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

impl Debug for KV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        f.write_str(&self.key)?;
        f.write_str("\": ")?;

        self.value.fmt(f)
    }
}

pub trait LogWriter {
    fn log(&mut self, level: &LogLevel, msg: &String, kvs: &Vec<KV>);
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

    fn log(&self, level: LogLevel, msg: String, kvs: Vec<KV>) {
        for w in self.log_writers.read().unwrap().iter() {
            w.lock().unwrap().log(&level, &msg, &kvs);
        }
    }
}

struct StdLogger {}

impl StdLogger {
    fn new() -> LogWriterType {
        Arc::new(Mutex::new(Self {}))
    }
}

impl LogWriter for StdLogger {
    fn log(&mut self, level: &LogLevel, msg: &String, kvs: &Vec<KV>) {
        println!("level: {:?}; msg: {}, KVs: {:?}", level, msg, kvs);
    }
}

#[macro_export]
macro_rules! kvs {
    ($($key:expr => $value:expr),+) => (
        Some(vec![$($crate::logger::KV::new($key.into(), $value),)+])
    )
}

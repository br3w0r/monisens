use lazy_static::lazy_static;
use std::{
    fmt::{Debug, Write},
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

#[derive(Debug, Clone)]
enum KVType<'kv> {
    Timestamp(Duration),
    Any(Arc<dyn std::fmt::Debug + 'kv>),
}

#[derive(Clone)]
pub struct KV<'kv> {
    key: String,
    value: KVType<'kv>,
}

impl<'kv> KV<'kv> {
    pub fn new<T: std::fmt::Debug + 'kv>(key: String, value: T) -> Self {
        Self {
            key,
            value: KVType::Any(Arc::new(value)),
        }
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

impl<'kv> Debug for KV<'kv> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        f.write_str(&self.key)?;
        f.write_str("\": ")?;

        self.value.fmt(f)
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

struct StdLogger {}

impl StdLogger {
    fn new() -> LogWriterType {
        Arc::new(Mutex::new(Self {}))
    }
}

impl LogWriter for StdLogger {
    fn log<'log>(&mut self, level: &'log LogLevel, msg: &'log str, kvs: &Vec<KV<'log>>) {
        println!("level: {:?}; msg: {}, KVs: {:?}", level, msg, kvs);
    }
}

#[macro_export]
macro_rules! kvs {
    ($($key:expr => $value:expr),+) => (
        Some(vec![$($crate::logger::KV::new($key.into(), $value),)+])
    )
}

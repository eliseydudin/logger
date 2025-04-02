#![feature(thread_id_value)]

use chrono::Utc;
use core::fmt;
use log::{Level, Log};
use std::{
    io::{self, Error, Write},
    sync::{Mutex, OnceLock},
    thread,
};

pub struct Logger {
    inner: Box<Mutex<dyn Write + Send>>,
}

impl fmt::Debug for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logger")
            .field("inner", &"Box(<locked>)")
            .finish()
    }
}

unsafe impl Send for Logger {}

impl Logger {
    pub fn new(s: impl Write + Send + 'static) -> Self {
        Self {
            inner: Box::new(Mutex::new(s)),
        }
    }

    pub fn replace_logger(s: impl Write + Send + 'static) {
        LOGGER
            .set(Logger::new(s))
            .expect("Replacing the logger failed")
    }

    fn unwrap(&self) -> Result<std::sync::MutexGuard<'_, dyn Write + Send + 'static>, Error> {
        self.inner
            .lock()
            .map_err(|_| Error::new(io::ErrorKind::Other, "Mutex poisoned!"))
    }

    pub fn init(s: impl Write + Send + 'static) {
        log::set_logger(LOGGER.get_or_init(|| Logger::new(s)))
            .expect("Cannot initialize the logger");
    }
}

impl Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.unwrap().map(|mut lock| lock.write(buf))?
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.unwrap().map(|mut lock| lock.flush())?
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn flush(&self) {
        match self.unwrap() {
            Ok(mut lock) => {
                let _ = lock.flush();
            }
            Err(_e) => self.inner.clear_poison(),
        }
    }

    fn log(&self, record: &log::Record) {
        const RESET_COLOR: &'static str = "\x1b[0m";

        let (color, label) = match record.level() {
            Level::Info => ("\x1B[97m", "INF"),
            Level::Debug => ("\x1B[36m", "DBG"),
            Level::Error => ("\x1B[31m", "ERR"),
            Level::Warn => ("\x1B[33m", "WRN"),
            Level::Trace => ("\x1B[97m", "TRC"),
        };

        let thr = thread::current();

        let thread = match thr.name() {
            Some(name) => {
                let mut s = name.to_owned();
                s.truncate(5);
                s
            }
            None => {
                let id = thr.id().as_u64();
                format!("id {id}")
            }
        };

        match self.unwrap() {
            Ok(mut lock) => {
                let _ = writeln!(
                    lock,
                    "{}{color}  <{thread}>\t[{label}]{RESET_COLOR} {}",
                    Utc::now().format("%H:%M:%S"),
                    record.args()
                );
            }
            Err(e) => panic!("Cannot write to the logger: {e}"),
        };
    }
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

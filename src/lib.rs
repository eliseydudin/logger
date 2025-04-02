#![feature(thread_id_value)]
#![feature(try_blocks)]

use chrono::Utc;
use core::fmt;
use log::{Level, Log};
use std::{
    io::{self, BufWriter},
    sync, thread,
};

pub struct Logger {
    inner: Box<sync::Mutex<dyn io::Write + Send>>,
}

impl fmt::Debug for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logger")
            .field("inner", &"<locked>")
            .finish()
    }
}

unsafe impl Send for Logger {}

impl Logger {
    pub fn new<F>(s: F) -> Self
    where
        F: io::Write + Send + 'static,
    {
        Self {
            inner: Box::new(sync::Mutex::new(s)),
        }
    }

    pub fn new_buffered<F>(file: F) -> Self
    where
        F: io::Write + Send + 'static,
    {
        let bufwriter = BufWriter::with_capacity(50, file);
        Self::new(bufwriter)
    }

    pub fn replace_logger<F>(s: F)
    where
        F: io::Write + Send + 'static,
    {
        LOGGER
            .set(Logger::new(s))
            .expect("Replacing the logger failed")
    }

    fn unwrap(&self) -> Result<sync::MutexGuard<'_, dyn io::Write + Send + 'static>, io::Error> {
        self.inner
            .lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Mutex poisoned!"))
    }

    pub fn init<F>(s: F)
    where
        F: io::Write + Send + 'static,
    {
        log::set_max_level(log::LevelFilter::Trace);
        log::set_logger(LOGGER.get_or_init(|| Logger::new(s)))
            .expect("Cannot initialize the logger");
    }

    pub fn init_buffered<F>(s: F)
    where
        F: io::Write + Send + 'static,
    {
        log::set_max_level(log::LevelFilter::Trace);
        log::set_logger(LOGGER.get_or_init(|| Logger::new_buffered(s)))
            .expect("Cannot initialize the logger");
    }
}

impl io::Write for Logger {
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

        let result: Result<(), io::Error> = try {
            let mut lock = self.unwrap()?;
            writeln!(
                lock,
                "{}{color} {thread} [{label}]{RESET_COLOR} {}",
                Utc::now().format("%H:%M:%S"),
                record.args()
            )?;
        };

        result.expect("Cannot write to the logger")
    }
}

static LOGGER: sync::OnceLock<Logger> = sync::OnceLock::new();

#[inline]
pub fn init<F>(f: F)
where
    F: io::Write + Send + 'static,
{
    Logger::init(f);
}

#[inline]
pub fn init_buffered<F>(f: F)
where
    F: io::Write + Send + 'static,
{
    Logger::init_buffered(f);
}

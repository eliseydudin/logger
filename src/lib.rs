use chrono::Utc;
use lazy_exclusive::LazyExclusive;
use log::{Level, Log};
use std::io::Write;

#[derive(Default)]
pub enum Inner {
    #[default]
    Stdout,
    Stderr,
    Buffer(Box<dyn Write>),
}

impl<T: Write + 'static> From<T> for Inner {
    fn from(value: T) -> Self {
        let boxed = Box::new(value);
        Self::Buffer(boxed)
    }
}

pub struct Logger {
    inner: LazyExclusive<Inner>,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            inner: LazyExclusive::default(),
        }
    }
}

impl Logger {
    pub const fn new() -> Self {
        Self {
            inner: LazyExclusive::new(Inner::Stdout),
        }
    }

    pub fn set_inner(&self, inner: Inner) {
        self.inner.swap(inner);
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn flush(&self) {
        let mut lock = self.inner.wait();
        match &mut *lock {
            Inner::Buffer(buffer) => {
                let _ = buffer.flush();
            }
            _ => (),
        };
    }

    fn log(&self, record: &log::Record) {
        let mut lock = self.inner.wait();

        const RESET_COLOR: &'static str = "\x1b[0m";
        let (color, label) = match record.level() {
            Level::Info => ("\x1B[97m", "INF"),
            Level::Debug => ("\x1B[36m", "DBG"),
            Level::Error => ("\x1B[31m", "ERR"),
            Level::Warn => ("\x1B[33m", "WRN"),
            Level::Trace => ("\x1B[97m", "TRC"),
        };

        let current = std::thread::current();
        let name = current.name().unwrap_or("unknown");

        match &mut *lock {
            Inner::Stdout => {
                println!(
                    "{}{color} {name} [{label}]{RESET_COLOR} {}",
                    Utc::now().format("%H:%M:%S"),
                    record.args()
                )
            }
            Inner::Stderr => eprintln!(
                "{}{color} {name} [{label}] {RESET_COLOR} {}",
                Utc::now().format("%H:%M:%S"),
                record.args()
            ),
            Inner::Buffer(buffer) => {
                let _ = writeln!(
                    buffer,
                    "{} {name} [{label}] {}",
                    Utc::now().format("%H:%M:%S"),
                    record.args()
                );
            }
        }
    }
}

static LOGGER: Logger = Logger::new();

pub fn get_logger() -> &'static Logger {
    &LOGGER
}

pub fn replace_logger<T>(inner: T)
where
    T: Into<Inner>,
{
    LOGGER.set_inner(inner.into());
}

pub fn init() -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER)
}

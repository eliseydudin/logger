use chrono::Utc;
use core::fmt;
use log::{Level, Log};
use std::{
    io,
    sync::{LazyLock, Mutex, MutexGuard},
};

enum Inner {
    Stdout,
    Stderr,
    Other(Box<dyn io::Write + Send>),
}

pub struct Logger(Mutex<Inner>);

impl fmt::Debug for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = match &*self.inner() {
            Inner::Stdout => "stdout",
            Inner::Stderr => "stderr",
            Inner::Other(_) => "<locked>",
        };

        f.debug_tuple("Logger").field(&inner).finish()
    }
}

impl Logger {
    pub fn new<B>(buffer: B) -> Self
    where
        B: io::Write + Send + 'static,
    {
        Self(Mutex::new(Inner::Other(Box::new(buffer))))
    }

    pub fn stdout() -> Self {
        Self(Mutex::new(Inner::Stdout))
    }

    pub fn stderr() -> Self {
        Self(Mutex::new(Inner::Stderr))
    }

    fn inner(&self) -> MutexGuard<'_, Inner> {
        match self.0.lock() {
            Ok(lock) => lock,
            Err(_) => {
                self.0.clear_poison();
                self.inner()
            }
        }
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn flush(&self) {
        let mut inner = self.inner();
        if let Inner::Other(inner) = &mut *inner {
            inner.flush().expect("Cannot flush the logger");
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

        let current = std::thread::current();
        let name = current.name().unwrap_or("unknown");

        let mut inner = self.inner();
        let inner_ref = &mut *inner;

        match inner_ref {
            Inner::Stdout => println!(
                "{}{color} {name} [{label}]{RESET_COLOR} {}",
                Utc::now().format("%H:%M:%S"),
                record.args()
            ),
            Inner::Other(mutex) => {
                let err = writeln!(
                    mutex,
                    "{} {name} [{label}] {}",
                    Utc::now().format("%H:%M:%S"),
                    record.args()
                );

                err.expect("Cannot write to lock")
            }
            Inner::Stderr => eprintln!(
                "{}{color} {name} [{label}] {RESET_COLOR} {}",
                Utc::now().format("%H:%M:%S"),
                record.args()
            ),
        };
    }
}

// impl Log for Logger {
//     fn enabled(&self, _metadata: &log::Metadata) -> bool {
//         true
//     }

//     fn flush(&self) {
//         match self.unwrap() {
//             Ok(mut lock) => {
//                 let _ = lock.flush();
//             }
//             Err(_e) => self.inner.clear_poison(),
//         }
//     }

//     fn log(&self, record: &log::Record) {
//         const RESET_COLOR: &'static str = "\x1b[0m";

//         let (color, label) = match record.level() {
//             Level::Info => ("\x1B[97m", "INF"),
//             Level::Debug => ("\x1B[36m", "DBG"),
//             Level::Error => ("\x1B[31m", "ERR"),
//             Level::Warn => ("\x1B[33m", "WRN"),
//             Level::Trace => ("\x1B[97m", "TRC"),
//         };

//         let thr = thread::current();

//         let thread = match thr.name() {
//             Some(name) => {
//                 let mut s = name.to_owned();
//                 s.truncate(5);
//                 s
//             }
//             None => {
//                 let id = thr.id().as_u64();
//                 format!("id {id}")
//             }
//         };

//         let result: Result<(), io::Error> = try {
//             let mut lock = self.unwrap()?;
//             writeln!(
//                 lock,
//                 "{}{color} {thread} [{label}]{RESET_COLOR} {}",
//                 Utc::now().format("%H:%M:%S"),
//                 record.args()
//             )?;
//         };

//         result.expect("Cannot write to the logger")
//     }
// }

static LOGGER: LazyLock<Logger> = LazyLock::new(|| Logger::stdout());

pub fn init_stdout() {
    log::set_logger(&*LOGGER).expect("Cannot initialize the logger!");
}

pub fn swap<B>(buffer: B)
where
    B: io::Write + Send + 'static,
{
    let mut inner = LOGGER.inner();
    *inner = Inner::Other(Box::new(buffer));
}

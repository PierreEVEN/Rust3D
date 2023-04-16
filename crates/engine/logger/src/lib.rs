use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::thread::{ThreadId};
use colored::{ColoredString, Colorize};
use std::env;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use time::OffsetDateTime;

pub trait Logger: Send {
    fn print(&mut self, message: &LogMessage);
}

#[derive(Default)]
pub enum LogSeverity {
    DEBUG(u32),
    #[default]
    INFO,
    WARNING,
    ERROR,
    FATAL,
}

impl LogSeverity {
    pub fn colorize(&self, text: &str) -> ColoredString {
        match self {
            LogSeverity::DEBUG(_) => {
                text.blue()
            }
            LogSeverity::INFO => {
                text.cyan()
            }
            LogSeverity::WARNING => {
                text.yellow()
            }
            LogSeverity::ERROR => {
                text.red()
            }
            LogSeverity::FATAL => {
                text.purple()
            }
        }
    }

    pub fn should_display_now(&self) -> bool {
        if let LogSeverity::DEBUG(required_level) = self {
            let env_var = match env::var("DEBUG_LEVEL") {
                Ok(var) => { var }
                Err(_) => { return false; }
            };
            if let Ok(value) = env_var.parse::<i32>() {
                if value >= 0 && *required_level > value as u32 {
                    return false;
                }
                return true;
            };
            return false;
        }
        true
    }
}

impl Display for LogSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LogSeverity::DEBUG(level) => { f.write_str(format!("D{level}").as_str()) }
            LogSeverity::INFO => { f.write_str("I") }
            LogSeverity::WARNING => { f.write_str("W") }
            LogSeverity::ERROR => { f.write_str("E") }
            LogSeverity::FATAL => { f.write_str("F") }
        }
    }
}

#[derive(Default)]
pub struct LogMessage {
    pub severity: LogSeverity,
    pub context: String,
    pub text: String,
    pub thread_id: Option<ThreadId>,
}

impl LogMessage {
    pub fn to_string_colored(&self) -> ColoredString {
        let thread_name = match self.thread_id {
            None => { "".to_string() }
            Some(thread_id) => { "::".to_string() + get_thread_name(thread_id).as_str() }
        };

        const DATE_FORMAT_STR: &'static str = "%Y-%m-%d][%H:%M:%S";
        let dt: OffsetDateTime = SystemTime::now().into();
        let dt2 = OffsetDateTime::now_utc();  // Also uses std::time (v0.2.26)

        println!("{}", dt.format(DATE_FORMAT_STR));
        println!("{}", dt2.format(DATE_FORMAT_STR));

        self.severity.colorize(format!("{:?} |{}| [{}{}] : {}",
                                       "",
                                       self.severity,
                                       self.context,
                                       thread_name,
                                       self.text
        ).as_str())
    }
}

lazy_static! {
    static ref LOGGERS: Mutex<Vec<Box<dyn Logger>>> = Mutex::new(vec![Box::new(StandardOutputLogger{})]);
    static ref THREAD_NAMES: Mutex<HashMap<ThreadId, String>> = Default::default();
}

pub fn set_thread_name(id: ThreadId, name: &str) {
    THREAD_NAMES.lock().expect("lock failed").insert(id, name.to_string());
}

pub fn get_thread_name(id: ThreadId) -> String {
    match THREAD_NAMES.lock().expect("lock failed").get(&id) {
        None => { "unregistered_thread_name".to_string() }
        Some(name) => { name.clone() }
    }
}

pub fn broadcast_log(message: LogMessage) {
    if !message.severity.should_display_now() { return; }
    for logger in &mut *LOGGERS.lock().expect("lock failed") {
        logger.print(&message);
    }
}

#[macro_export]
macro_rules! debug {
    ($level:expr, $($fmt_args:tt)*) => ({
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::DEBUG($level),
            thread_id: None,
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args)*)
        });
    })
}

#[macro_export]
macro_rules! info {
    ($($fmt_args:tt)*) => ({
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::INFO,
            thread_id: None,
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args)*)
        });
    })
}


#[macro_export]
macro_rules! warning {
    ($($fmt_args:tt)*) => ({
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::WARNING,
            thread_id: None,
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args)*)
        });
    })
}


#[macro_export]
macro_rules! error {
    ($($fmt_args:tt)*) => ({
        #[cfg(not(debug_assertions))]
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::ERROR,
            thread_id: None,
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args)*),
        });
        #[cfg(debug_assertions)]
        {
            use std::backtrace::Backtrace;
            $crate::broadcast_log($crate::LogMessage {
                severity: $crate::LogSeverity::ERROR,
                thread_id: None,
                context: env!("CARGO_PKG_NAME").to_string(),
                text: format!("{}\n{}", format!($($fmt_args)*), Backtrace::capture()),
            });
        }
    })
}

#[macro_export]
macro_rules! fatal {
    ($($fmt_args:tt)*) => ({
        #[cfg(not(debug_assertions))]
        {
            $crate::broadcast_log($crate::LogMessage {
                severity: $crate::LogSeverity::FATAL,
                thread_id: None,
                context: env!("CARGO_PKG_NAME").to_string(),
                text: format!($($fmt_args)*),
            });
            panic!($($fmt_args)*);
        }
        #[cfg(debug_assertions)]
        {
            use std::backtrace::Backtrace;
            $crate::broadcast_log($crate::LogMessage {
                severity: $crate::LogSeverity::FATAL,
                thread_id: None,
                context: env!("CARGO_PKG_NAME").to_string(),
                text: format!("{}\n{}", format!($($fmt_args)*), Backtrace::capture()),
            });
            std::process::exit(-1);
        }
    })
}
pub struct StandardOutputLogger {}

impl Logger for StandardOutputLogger {
    fn print(&mut self, message: &LogMessage) {
        println!("{}", message.to_string_colored());
    }
}
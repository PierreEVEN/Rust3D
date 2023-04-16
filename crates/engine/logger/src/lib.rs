use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use std::thread::{ThreadId};
use std::{env, thread};
use std::fmt::{Display, Formatter};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub trait Logger: Send {
    fn print(&mut self, message: &LogMessage, backtrace: &str);
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
    pub fn color(&self) -> ColorSpec {
        let mut col = ColorSpec::new();
        match self {
            LogSeverity::DEBUG(_) => {
                col.set_fg(Some(Color::Blue))
            }
            LogSeverity::INFO => {
                col.set_fg(Some(Color::Cyan))
            }
            LogSeverity::WARNING => {
                col.set_fg(Some(Color::Yellow))
            }
            LogSeverity::ERROR => {
                col.set_fg(Some(Color::Red))
            }
            LogSeverity::FATAL => {
                col.set_fg(Some(Color::Magenta))
            }
        };
        col
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

impl Display for LogMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let thread_name = match self.thread_id {
            None => { "".to_string() }
            Some(thread_id) => { "::".to_string() + get_thread_label(thread_id).as_str() }
        };
        f.write_str(
        format!("{} |{}| [{}{}] : {}",
                chrono::Utc::now().format("%H:%M:%S"),
                self.severity,
                self.context,
                thread_name,
                self.text
        ).as_str())
    }
}

lazy_static! {
    static ref LOGGERS: Mutex<Vec<Box<dyn Logger>>> = Mutex::new(vec![Box::new(StandardOutputLogger{})]);
    static ref THREAD_LABELS: Mutex<HashMap<ThreadId, String>> = Default::default();
}

pub fn set_main_thread() {
    set_thread_label(thread::current().id(), "main thread");
}

pub fn set_thread_label(id: ThreadId, name: &str) {
    THREAD_LABELS.lock().expect("lock failed").insert(id, name.to_string());
}

pub fn get_thread_label(id: ThreadId) -> String {
    match THREAD_LABELS.lock().expect("lock failed").get(&id) {
        None => { "unregistered_thread_name".to_string() }
        Some(name) => { name.clone() }
    }
}

pub fn broadcast_log(message: LogMessage, backtrace: String) {
    if !message.severity.should_display_now() { return; }
    for logger in &mut *LOGGERS.lock().expect("lock failed") {
        logger.print(&message, &backtrace);
    }
}

#[macro_export]
macro_rules! debug {
    ($level:expr, $($fmt_args:tt)*) => ({
        use std::thread;
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::DEBUG($level),
            thread_id: Some(thread::current().id()),
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args)*)
        }, "".to_string());
    })
}

#[macro_export]
macro_rules! info {
    ($($fmt_args:tt)*) => ({
        use std::thread;
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::INFO,
            thread_id: Some(thread::current().id()),
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args)*)
        }, "".to_string());
    })
}

#[macro_export]
macro_rules! warning {
    ($($fmt_args:tt)*) => ({
        use std::thread;
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::WARNING,
            thread_id: Some(thread::current().id()),
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args)*)
        }, "".to_string());
    })
}

#[macro_export]
macro_rules! error {
    ($($fmt_args:tt)*) => ({
        use std::thread;
        #[cfg(not(debug_assertions))]
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::ERROR,
            thread_id: Some(thread::current().id()),
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args)*),
        }, "".to_string());
        #[cfg(debug_assertions)]
        {
            use std::backtrace::Backtrace;
            $crate::broadcast_log($crate::LogMessage {
                severity: $crate::LogSeverity::ERROR,
                thread_id: Some(thread::current().id()),
                context: env!("CARGO_PKG_NAME").to_string(),
                text: format!($($fmt_args)*),
            },
            Backtrace::force_capture().to_string()
            );
        }
    })
}

#[macro_export]
macro_rules! fatal {
    ($($fmt_args:tt)*) => ({
        use std::thread;
        #[cfg(not(debug_assertions))]
        {
            $crate::broadcast_log($crate::LogMessage {
                severity: $crate::LogSeverity::FATAL,
                thread_id: Some(thread::current().id()),
                context: env!("CARGO_PKG_NAME").to_string(),
                text: format!($($fmt_args)*),
            }, "".to_string());
            panic!($($fmt_args)*);
        }
        #[cfg(debug_assertions)]
        {
            use std::backtrace::Backtrace;
            $crate::broadcast_log($crate::LogMessage {
                severity: $crate::LogSeverity::FATAL,
                thread_id: Some(thread::current().id()),
                context: env!("CARGO_PKG_NAME").to_string(),
                text: format!($($fmt_args)*),
            },
            Backtrace::force_capture().to_string()
            );
            std::process::exit(-1);
        }
    })
}
pub struct StandardOutputLogger {}

impl Logger for StandardOutputLogger {
    fn print(&mut self, message: &LogMessage, backtrace: &str) {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);

        stdout.set_color(&message.severity.color()).unwrap();

        writeln!(&mut stdout, "{}", message).unwrap();
        if !backtrace.is_empty() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Black)).set_bg(Some(Color::Rgb(150, 150, 150)))).unwrap();
            writeln!(&mut stdout, "\n{}", backtrace).unwrap();
        }
        
        stdout.flush().unwrap();
        stdout.set_color(&ColorSpec::default()).unwrap();
    }
}
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread::ThreadId;
use std::{env, fs, thread};

use chrono::DateTime;
use lazy_static::lazy_static;
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
            LogSeverity::DEBUG(_) => col.set_fg(Some(Color::Blue)),
            LogSeverity::INFO => col.set_fg(Some(Color::Cyan)),
            LogSeverity::WARNING => col.set_fg(Some(Color::Yellow)),
            LogSeverity::ERROR => col.set_fg(Some(Color::Red)),
            LogSeverity::FATAL => col.set_fg(Some(Color::Magenta)),
        };
        col
    }

    pub fn should_display_now(&self) -> bool {
        if let LogSeverity::DEBUG(required_level) = self {
            let env_var = match env::var("DEBUG_LEVEL") {
                Ok(var) => var,
                Err(_) => {
                    return false;
                }
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
            LogSeverity::DEBUG(level) => f.write_str(format!("D{level}").as_str()),
            LogSeverity::INFO => f.write_str("I"),
            LogSeverity::WARNING => f.write_str("W"),
            LogSeverity::ERROR => f.write_str("E"),
            LogSeverity::FATAL => f.write_str("F"),
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
            None => "".to_string(),
            Some(thread_id) => "::".to_string() + get_thread_label(thread_id).as_str(),
        };
        f.write_str(
            format!(
                "{} |{}| [{}{}] : {}",
                chrono::Utc::now().format("%H:%M:%S"),
                self.severity,
                self.context,
                thread_name,
                self.text
            )
            .as_str(),
        )
    }
}

lazy_static! {
    static ref LOGGERS: Mutex<Vec<Box<dyn Logger>>> =
        Mutex::new(vec![Box::new(StandardOutputLogger {})]);
    static ref THREAD_LABELS: Mutex<HashMap<ThreadId, String>> = Default::default();
}

pub fn set_main_thread() {
    set_thread_label(thread::current().id(), "main_thread");
}

pub fn set_thread_label(id: ThreadId, name: &str) {
    THREAD_LABELS
        .lock()
        .expect("lock failed")
        .insert(id, name.to_string());
}

pub fn get_thread_label(id: ThreadId) -> String {
    match THREAD_LABELS.lock().expect("lock failed").get(&id) {
        None => "unregistered_thread_name".to_string(),
        Some(name) => name.clone(),
    }
}

pub fn bind_logger(logger: Box<dyn Logger>) {
    LOGGERS.lock().expect("failed to lock").push(logger);
}

pub fn broadcast_log(message: LogMessage, backtrace: String) {
    if !message.severity.should_display_now() {
        return;
    }
    for logger in &mut *LOGGERS.lock().expect("lock failed") {
        logger.print(&message, &backtrace);
    }
}

#[macro_export]
macro_rules! debug {
    ($level:expr, $($fmt_args:expr), *) => ({
        use std::thread;
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::DEBUG($level),
            thread_id: Some(thread::current().id()),
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args), *)
        }, "".to_string());
    });
    ($($fmt_args:expr), *) => ({
        use std::thread;
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::DEBUG(0),
            thread_id: Some(thread::current().id()),
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args), *)
        }, "".to_string());
    })
}

#[macro_export]
macro_rules! info {
    ($($fmt_args:expr), *) => ({
        use std::thread;
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::INFO,
            thread_id: Some(thread::current().id()),
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args), *)
        }, "".to_string());
    })
}

#[macro_export]
macro_rules! warning {
    ($($fmt_args:expr), *) => ({
        use std::thread;
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::WARNING,
            thread_id: Some(thread::current().id()),
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args), *)
        }, "".to_string());
    })
}

#[macro_export]
macro_rules! error {
    ($($fmt_args:expr), *) => ({
        use std::thread;
        #[cfg(not(debug_assertions))]
        $crate::broadcast_log($crate::LogMessage {
            severity: $crate::LogSeverity::ERROR,
            thread_id: Some(thread::current().id()),
            context: env!("CARGO_PKG_NAME").to_string(),
            text: format!($($fmt_args), *),
        }, "".to_string());
        #[cfg(debug_assertions)]
        {
            use std::backtrace::Backtrace;
            $crate::broadcast_log($crate::LogMessage {
                severity: $crate::LogSeverity::ERROR,
                thread_id: Some(thread::current().id()),
                context: env!("CARGO_PKG_NAME").to_string(),
                text: format!($($fmt_args), *),
            },
            Backtrace::force_capture().to_string()
            );
        }
    })
}

#[macro_export]
macro_rules! fatal {
    ($($fmt_args:expr), *) => ({
        use std::thread;
        #[cfg(not(debug_assertions))]
        {
            $crate::broadcast_log($crate::LogMessage {
                severity: $crate::LogSeverity::FATAL,
                thread_id: Some(thread::current().id()),
                context: env!("CARGO_PKG_NAME").to_string(),
                text: format!($($fmt_args), *),
            }, "".to_string());
            panic!($($fmt_args), *);
        }
        #[cfg(debug_assertions)]
        {
            use std::backtrace::Backtrace;
            $crate::broadcast_log($crate::LogMessage {
                severity: $crate::LogSeverity::FATAL,
                thread_id: Some(thread::current().id()),
                context: env!("CARGO_PKG_NAME").to_string(),
                text: format!($($fmt_args), *),
            },
            Backtrace::force_capture().to_string()
            );
            std::process::exit(-1);
        }
    })
}

#[macro_export]
macro_rules! init {
    () => {{
        $crate::set_main_thread();
        $crate::bind_logger(Box::new($crate::FileLogger::new(
            std::path::Path::new("saved/log/"),
            env!("CARGO_PKG_NAME"),
        )));
    }};
}

pub struct StandardOutputLogger {}

impl Logger for StandardOutputLogger {
    fn print(&mut self, message: &LogMessage, backtrace: &str) {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);

        stdout.set_color(&message.severity.color()).unwrap();

        writeln!(&mut stdout, "{}", message).unwrap();
        if !backtrace.is_empty() {
            stdout
                .set_color(
                    ColorSpec::new()
                        .set_fg(Some(Color::Black))
                        .set_bg(Some(Color::Rgb(150, 150, 150))),
                )
                .unwrap();
            writeln!(&mut stdout, "\n{}", backtrace).unwrap();
        }
        stdout.set_color(&ColorSpec::default()).unwrap();
        stdout.flush().unwrap();
    }
}

pub struct FileLogger {
    file: File,
    path: String,
}

impl FileLogger {
    pub fn new(save_path: &Path, log_file_name: &str) -> Self {
        let mut log_full_path = save_path.join(log_file_name);
        log_full_path.set_extension("log");
        if log_full_path.exists() {
            let dt: DateTime<chrono::Utc> =
                log_full_path.metadata().unwrap().modified().unwrap().into();

            let moved_log_file_name = PathBuf::from(format!(
                "{log_file_name}_{}",
                dt.format("%Y.%m.%d-%H.%M.%S")
            ))
            .to_str()
            .unwrap()
            .to_string();
            let mut iter = 0;
            loop {
                let moved_log_file_name = PathBuf::from(if iter < 1 {
                    format!("{}.log", moved_log_file_name)
                } else {
                    format!("{moved_log_file_name}({iter}).log")
                });

                let new_path = save_path.join(moved_log_file_name);
                if new_path.as_path().exists() {
                    iter += 1;
                    continue;
                }
                fs::rename(log_full_path.clone(), new_path.as_path()).unwrap_or_else(|error| {
                    error!(
                        "Failed to rename {:?} to {:?} : {error}",
                        log_full_path,
                        new_path.as_path()
                    )
                });
                break;
            }
        }
        assert!(!log_full_path.exists());
        let file = {
            fs::create_dir_all(save_path).unwrap();
            File::create(log_full_path.clone())
        }
        .unwrap_or_else(|_| fatal!("failed to create file {:?}", log_full_path));
        Self {
            file,
            path: save_path.to_str().unwrap().to_string(),
        }
    }
}

impl Logger for FileLogger {
    fn print(&mut self, message: &LogMessage, backtrace: &str) {
        self.file
            .write_all(message.to_string().as_bytes())
            .unwrap_or_else(|_| error!("Failed to write to {}", self.path));
        if !backtrace.is_empty() {
            self.file
                .write_all(format!("\n{backtrace}").as_bytes())
                .unwrap_or_else(|_| error!("Failed to write to {}", self.path));
        }
        self.file.write_all("\n".as_bytes()).unwrap();
        self.file.flush().expect("failed to flush file");
    }
}

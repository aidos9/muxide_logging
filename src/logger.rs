//! Loggers useful for various types of logging.

use crate::format::Format;
use crate::log::{LogItem, LogLevel, Logger};
use chrono::{DateTime, Local, TimeZone, Utc};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
/// The default logger, writes any new logs to a file by appending.
///
/// ## Using FileLogger
/// `FileLogger` can be used through the [error!], [warning!], [state_change!] and [info!] macros
/// which utilise the [Logger] trait. However it can also be used manually.
///
/// ```no_run
/// use muxide_logging::logger::FileLogger;
/// use muxide_logging::log::{Logger, LogItem, LogLevel};
/// use muxide_logging::format::Format;
/// use chrono::Local;
///
/// let mut logger = FileLogger::<Local>::new();
/// logger.open_file("file_name").unwrap();
/// logger.log_item(LogItem::new(Format::<Local>::default(), LogLevel::Information, "Log message"));
/// ```
///
pub struct FileLogger<Tz: TimeZone>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy,
{
    /// The file to write to. We have an optional value so that the user can open a file on demand.
    file: Option<File>,
    /// Whether we should panic on IO errors or ignore them.
    panic_on_fail: bool,
    /// A custom Format to use as an override.
    override_format: Option<Format<Tz>>,
    /// Any logs with these log levels will be ignored.
    restricted_log_levels: Vec<LogLevel>,
}

#[derive(Clone, Debug)]
/// An alternative logger, primarily used for testing purposes. However instead of retuning nothing
/// it will return a string when using each logging macro.
///
/// ## Using StringLogger
/// `FileLogger` can be used through the [error!], [warning!], [state_change!] and [info!] macros
/// which utilise the [Logger] trait. However it can also be used manually.
///
/// ```
/// use muxide_logging::logger::StringLogger;
/// use muxide_logging::log::{Logger, LogItem, LogLevel};
/// use muxide_logging::format::{Format, FormatItem};
/// use chrono::Local;
///
/// let mut logger = StringLogger::new();
/// let result = logger.log_item(LogItem::new(
///                 Format::new().append(FormatItem::LogString),
///                 LogLevel::Information,
///                 "Log message"));
/// assert_eq!(result, "Log message");
/// ```
///
pub struct StringLogger<Tz: TimeZone>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy,
{
    override_format: Option<Format<Tz>>,
}

impl<Tz: TimeZone> FileLogger<Tz>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy,
{
    /// Create a new instance of [FileLogger].
    pub fn new() -> FileLogger<Tz> {
        return Self {
            file: None,
            panic_on_fail: false,
            override_format: None,
            restricted_log_levels: Vec::new(),
        };
    }

    /// Sets whether a failed write to a file should result in a panic. By default this behaviour is
    /// disabled.
    pub fn set_panic_on_fail(&mut self, b: bool) {
        self.panic_on_fail = b;
    }

    /// Override any format supplied to the [log_item](Logger::log_item) method. This format is not used instead of
    /// the one supplied instead it is merged, selecting any values that are set but preferring
    /// values from the overridden format.
    pub fn set_override(&mut self, override_format: Format<Tz>) {
        self.override_format = Some(override_format);
    }

    /// Open a file for logging in append mode, creating a new one if it doesn't exist.
    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        self.file = Some(OpenOptions::new().append(true).create(true).open(path)?);

        return Ok(());
    }

    /// Close the file that is currently open.
    pub fn close_file(&mut self) {
        self.file = None;
    }

    /// Prevent logging any messages with these log levels
    pub fn restrict_log_levels(&mut self, levels: &[LogLevel]) {
        for level in levels {
            if !self.restricted_log_levels.contains(level) {
                self.restricted_log_levels.push(*level);
            }
        }
    }

    /// Allow any previously restricted log level.
    pub fn allow_log_levels(&mut self, levels: &[LogLevel]) {
        for level in levels {
            if let Some(idx) = self.restricted_log_levels.iter().position(|l| level == l) {
                self.restricted_log_levels.remove(idx);
            }
        }
    }
}

impl Logger for FileLogger<Local> {
    type ReturnType = ();

    fn can_log_item<Tz: TimeZone>(&self, item: &LogItem<Tz>) -> bool
    where
        Tz::Offset: std::fmt::Display,
        DateTime<Local>: From<DateTime<Tz>>,
        DateTime<Utc>: From<DateTime<Tz>>,
        DateTime<Tz>: Copy,
    {
        return !self.restricted_log_levels.contains(&item.level());
    }

    fn log_item<T: TimeZone>(&mut self, item: LogItem<T>) -> Self::ReturnType
    where
        T::Offset: std::fmt::Display,
        DateTime<Local>: From<DateTime<T>>,
        DateTime<Utc>: From<DateTime<T>>,
        DateTime<T>: Copy,
    {
        if let Some(file) = &mut self.file {
            let text = match self.override_format.as_ref() {
                Some(format) => {
                    let new_format = Format::<Local>::merged(format, item.format());

                    new_format.build_string(item.level(), &item.into_message())
                }
                None => item.into(),
            };

            let res = writeln!(file, "{}", text);

            if self.panic_on_fail {
                res.unwrap()
            }

            let res = file.flush();

            if self.panic_on_fail {
                res.unwrap();
            }
        }
    }
}

impl Logger for FileLogger<Utc> {
    type ReturnType = ();

    fn can_log_item<Tz: TimeZone>(&self, item: &LogItem<Tz>) -> bool
    where
        Tz::Offset: std::fmt::Display,
        DateTime<Local>: From<DateTime<Tz>>,
        DateTime<Utc>: From<DateTime<Tz>>,
        DateTime<Tz>: Copy,
    {
        return !self.restricted_log_levels.contains(&item.level());
    }

    fn log_item<T: TimeZone>(&mut self, item: LogItem<T>) -> Self::ReturnType
    where
        T::Offset: std::fmt::Display,
        DateTime<Local>: From<DateTime<T>>,
        DateTime<Utc>: From<DateTime<T>>,
        DateTime<T>: Copy,
    {
        if let Some(file) = &mut self.file {
            let text = match self.override_format.as_ref() {
                Some(format) => {
                    let new_format = Format::<Utc>::merged(format, item.format());

                    new_format.build_string(item.level(), &item.into_message())
                }
                None => item.into(),
            };

            let res = writeln!(file, "{}", text);

            if self.panic_on_fail {
                res.unwrap()
            }

            let res = file.flush();

            if self.panic_on_fail {
                res.unwrap();
            }
        }
    }
}

impl StringLogger<Local> {
    pub fn new() -> Self {
        return Self::new_tz();
    }
}

impl<Tz: TimeZone> StringLogger<Tz>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy,
{
    /// Create a new instance of [StringLogger] with a custom Tz.
    pub fn new_tz() -> Self {
        return Self {
            override_format: None,
        };
    }

    /// Override any format supplied to the [log_item](Logger::log_item) method. This format is not used instead of
    /// the one supplied instead it is merged, selecting any values that are set but preferring
    /// values from the overridden format.
    pub fn set_override(&mut self, format: Format<Tz>) {
        self.override_format = Some(format);
    }
}

impl Logger for StringLogger<Local> {
    /// [StringLogger] returns the formatted [String] instead of nothing.
    type ReturnType = String;

    fn log_item<T: TimeZone>(&mut self, item: LogItem<T>) -> Self::ReturnType
    where
        T::Offset: std::fmt::Display,
        DateTime<Local>: From<DateTime<T>>,
        DateTime<Utc>: From<DateTime<T>>,
        DateTime<T>: Copy,
    {
        return match self.override_format.as_ref() {
            Some(format) => {
                let new_format = Format::<Local>::merged(format, item.format());

                new_format.build_string(item.level(), &item.into_message())
            }
            None => item.into(),
        };
    }
}

impl Logger for StringLogger<Utc> {
    /// [StringLogger] returns the formatted [String] instead of nothing.
    type ReturnType = String;

    fn log_item<T: TimeZone>(&mut self, item: LogItem<T>) -> Self::ReturnType
    where
        T::Offset: std::fmt::Display,
        DateTime<Local>: From<DateTime<T>>,
        DateTime<Utc>: From<DateTime<T>>,
        DateTime<T>: Copy,
    {
        return match self.override_format.as_ref() {
            Some(format) => {
                let new_format = Format::<Utc>::merged(format, item.format());

                new_format.build_string(item.level(), &item.into_message())
            }
            None => item.into(),
        };
    }
}

//! Loggers useful for various types of logging.

use crate::format::Format;
use crate::log::{LogItem, Logger};
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
///
/// let mut logger = FileLogger::new();
/// logger.open_file("file_name").unwrap();
/// logger.log_item(LogItem::new(Format::default(), LogLevel::Information, "Log message"));
/// ```
///
pub struct FileLogger {
    /// The file to write to. We have an optional value so that the user can open a file on demand.
    file: Option<File>,
    /// Whether we should panic on IO errors or ignore them.
    panic_on_fail: bool,
    /// A custom Format to use as an override.
    override_format: Option<Format>,
}

#[derive(Clone, Debug, PartialEq)]
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
///
/// let mut logger = StringLogger::new();
/// let result = logger.log_item(LogItem::new(
///                 Format::new().append(FormatItem::LogString),
///                 LogLevel::Information,
///                 "Log message"));
/// assert_eq!(result, "Log message");
/// ```
///
pub struct StringLogger {
    override_format: Option<Format>,
}

impl FileLogger {
    /// Create a new instance of [FileLogger].
    pub const fn new() -> Self {
        return Self {
            file: None,
            panic_on_fail: false,
            override_format: None,
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
    pub fn set_override(&mut self, override_format: Format) {
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
}

impl Logger for FileLogger {
    type ReturnType = ();

    fn log_item(&mut self, item: LogItem) -> Self::ReturnType {
        if let Some(file) = &mut self.file {
            let text = match self.override_format.as_ref() {
                Some(format) => {
                    let new_format = Format::merged(format, item.format());

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

impl StringLogger {
    /// Create a new instance of [StringLogger].
    pub const fn new() -> Self {
        return Self {
            override_format: None,
        };
    }

    /// Override any format supplied to the [log_item](Logger::log_item) method. This format is not used instead of
    /// the one supplied instead it is merged, selecting any values that are set but preferring
    /// values from the overridden format.
    pub fn set_override(&mut self, format: Format) {
        self.override_format = Some(format);
    }
}

impl Logger for StringLogger {
    /// [StringLogger] returns the formatted [String] instead of nothing.
    type ReturnType = String;

    fn log_item(&mut self, item: LogItem) -> Self::ReturnType {
        return match self.override_format.as_ref() {
            Some(format) => {
                let new_format = Format::merged(format, item.format());

                new_format.build_string(item.level(), &item.into_message())
            }
            None => item.into(),
        };
    }
}

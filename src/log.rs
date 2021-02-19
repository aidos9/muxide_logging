//! Log information

use crate::format::Format;
use chrono::{DateTime, Local, TimeZone, Utc};
use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
/// The level of severity of a log message.
pub enum LogLevel {
    Error,
    Warning,
    StateChange,
    Information,
}

#[derive(Clone, Debug)]
/// This item is used to dictate a log, it is used for the [Logger] trait to dictate the format,
/// level and content of a new log.
pub struct LogItem<Tz: TimeZone>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy,
{
    format: Format<Tz>,
    message: String,
    level: LogLevel,
}

/// Defines the expected behaviour of a logger. It is required for using any of the macros
/// supplied by this library.
pub trait Logger {
    /// The return type from performing [log_item](Logger::log_item).
    type ReturnType;

    /// Returns true if logging an item is permitted. By default this is true but when implementing
    /// a custom logger it may be useful to decline logging an item before the
    /// [log_item](Logger::log_item) method is called.
    fn can_log_item<Tz: TimeZone>(&self, _item: &LogItem<Tz>) -> bool
    where
        Tz::Offset: std::fmt::Display,
        DateTime<Tz>: Copy,
    {
        return true;
    }

    /// This method should log an item.
    fn log_item<Tz: TimeZone>(&mut self, item: LogItem<Tz>) -> Self::ReturnType
    where
        Tz::Offset: std::fmt::Display,
        DateTime<Local>: From<DateTime<Tz>>,
        DateTime<Utc>: From<DateTime<Tz>>,
        DateTime<Tz>: Copy;
}

impl LogLevel {
    /// Converts a [LogLevel] variant into a string.
    pub const fn as_str(&self) -> &'static str {
        return match self {
            LogLevel::Error => "Error",
            LogLevel::Warning => "Warning",
            LogLevel::StateChange => "StateChange",
            LogLevel::Information => "Information",
        };
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.as_str());
    }
}

impl<Tz: TimeZone> LogItem<Tz>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Local>: From<DateTime<Tz>>,
    DateTime<Utc>: From<DateTime<Tz>>,
    DateTime<Tz>: Copy,
{
    /// Create a new [LogItem].
    pub fn new(format: Format<Tz>, level: LogLevel, message: &str) -> Self {
        return Self {
            format,
            message: message.to_string(),
            level,
        };
    }

    /// Get the log level of this log.
    pub fn level(&self) -> LogLevel {
        return self.level;
    }

    /// Get the log message of this log.
    pub fn message(&self) -> &String {
        return &self.message;
    }

    /// Consume a [LogItem], returning the message.
    pub fn into_message(self) -> String {
        return self.message;
    }

    /// Get the format of this log.
    pub fn format(&self) -> &Format<Tz> {
        return &self.format;
    }
}

impl<Tz: TimeZone> Into<String> for LogItem<Tz>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy,
{
    /// Builds the format and returns the built string.
    fn into(self) -> String {
        return self.format.build_string(self.level, &self.message);
    }
}

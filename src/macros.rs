use crate::format::Format;
use crate::log::{LogItem, LogLevel, Logger};
use crate::DefaultLogger;
use chrono::{DateTime, Local, TimeZone, Utc};
use std::ops::DerefMut;

#[macro_export]
/// Logs a message at the error log level.
///
/// # Usage
/// Basic usage
/// ```no_run
/// use muxide_logging::error;
///
/// error!("my error message");
/// ```
///
/// With a custom logger
/// ```ignore
/// use muxide_logging::error;
///
/// error!("my error message", my_logger)
/// ```
macro_rules! error {
    ($message:expr) => {
        $crate::log_message!($crate::log::LogLevel::Error, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::log::LogLevel::Error, $message, $logger)
    };
}

#[macro_export]
/// Logs a message at the warning log level.
///
/// # Usage
/// Basic usage
/// ```no_run
/// use muxide_logging::warning;
///
/// warning!("my warning message");
/// ```
///
/// With a custom logger
/// ```ignore
/// use muxide_logging::warning;
///
/// warning!("my warning message", my_logger)
/// ```
macro_rules! warning {
    ($message:expr) => {
        $crate::log_message!($crate::log::LogLevel::Warning, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::log::LogLevel::Warning, $message, $logger)
    };
}

#[macro_export]
/// Logs a message at the state change log level.
///
/// # Usage
/// Basic usage
/// ```no_run
/// use muxide_logging::state_change;
///
/// state_change!("my error message");
/// ```
///
/// With a custom logger
/// ```ignore
/// use muxide_logging::state_change;
///
/// state_change!("my error message", my_logger)
/// ```
macro_rules! state_change {
    ($message:expr) => {
        $crate::log_message!($crate::log::LogLevel::StateChange, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::log::LogLevel::StateChange, $message, $logger)
    };
}

#[macro_export]
/// Logs a message at the information log level.
///
/// # Usage
/// Basic usage
/// ```no_run
/// use muxide_logging::info;
///
/// info!("my info message");
/// ```
///
/// With a custom logger
/// ```ignore
/// use muxide_logging::info;
///
/// info!("my info message", my_logger)
/// ```
macro_rules! info {
    ($message:expr) => {
        $crate::log_message!($crate::log::LogLevel::Information, $message)
    };

    ($message:expr, $logger:expr) => {
        $crate::log_message!($crate::log::LogLevel::Information, $message, $logger)
    };
}

#[macro_export]
/// Creates the default [Format] with populated line, column and module_path values based on the
/// location where this macro was called.
macro_rules! default_format {
    () => {
        $crate::format::Format::default()
            .set_column(column!() as usize)
            .set_line(line!() as usize)
            .set_module_path(module_path!())
            .set_file(file!())
    };
}

#[macro_export]
/// Creates the default [Format] with populated line, column and module_path values based on the
/// location where this macro was called for a custom timezone, either specified or otherwise.
///
/// # Usage
/// Automatic detection
/// ```no_run
/// use chrono::Utc;
/// use muxide_logging::format::Format;
/// use muxide_logging::default_format_custom_tz;
///
/// let utc_format: Format<Utc> = default_format_custom_tz!();
/// ```
///
/// Specified timezone
/// ```no_run
/// use chrono::Utc;
/// use muxide_logging::format::Format;
/// use muxide_logging::default_format_custom_tz;
///
/// let utc_format = default_format_custom_tz!(Utc);
/// ```
macro_rules! default_format_custom_tz {
    () => {
        $crate::format::Format::default_tz()
            .set_column(column!() as usize)
            .set_line(line!() as usize)
            .set_module_path(module_path!())
            .set_file(file!())
    };

    ($tz:ty) => {
        $crate::format::Format::<$tz>::default_tz()
            .set_column(column!() as usize)
            .set_line(line!() as usize)
            .set_module_path(module_path!())
            .set_file(file!())
    };
}

#[macro_export]
/// Builds a new [Format] based on a series of [FormatItem](crate::format::FormatItem) objects.
///
/// # Usage
/// ```
/// use muxide_logging::build_format_from_items;
/// use muxide_logging::format::{FormatItem, Format};
///
/// assert_eq!(
///     build_format_from_items!(FormatItem::LogLevel, FormatItem::LogString),
///     Format::new().append(FormatItem::LogLevel).append(FormatItem::LogString)
/// );
/// ```
macro_rules! build_format_from_items {
    ($($item:expr),*) => {
        $crate::format::Format::new()$(.append($item))*
    }
}

#[macro_export]
/// Builds a new [Format] based on a series of [FormatItem](crate::format::FormatItem) objects. With
/// a custom timezone.
///
/// # Usage
/// ```
/// use chrono::Utc;
/// use muxide_logging::build_format_from_items_tz;
/// use muxide_logging::format::{FormatItem, Format};
///
/// assert_eq!(
///     build_format_from_items_tz!(FormatItem::LogLevel, FormatItem::LogString),
///     Format::<Utc>::new_tz().append(FormatItem::LogLevel).append(FormatItem::LogString)
/// );
/// ```
macro_rules! build_format_from_items_tz {
    ($($item:expr),*) => {
        $crate::format::Format::new_tz()$(.append($item))*
    };
}

#[macro_export]
/// Helper macro for logging a message to a logger.
macro_rules! log_message {
    ($log_level:expr, $message:expr, $format:expr, $logger:expr) => {
        $crate::__log_message($log_level, $message, $format, &mut $logger);
    };

    ($log_level:expr, $message:expr, $logger:expr) => {
        $crate::log_message!($log_level, $message, $crate::default_format!(), $logger);
    };

    ($log_level:expr, $message:expr) => {
        $crate::__default_log_message($log_level, $message, $crate::default_format!());
    };
}

#[doc(hidden)]
/// A wrapper for __log_message that tries to lock the default logger.
pub fn __default_log_message<S: AsRef<str>, Tz: TimeZone>(
    log_level: LogLevel,
    message: S,
    format: Format<Tz>,
) -> Option<<DefaultLogger as Logger>::ReturnType>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Local>: From<DateTime<Tz>>,
    DateTime<Utc>: From<DateTime<Tz>>,
    DateTime<Tz>: Copy,
{
    if let Ok(mut logger) = crate::DEFAULT_LOGGER.lock() {
        return __log_message(log_level, message, format, logger.deref_mut());
    } else {
        return None;
    }
}

#[doc(hidden)]
/// Internal method used to write the log message to a file. We need a method instead of including
/// the macro because 'let' variables are not supported in some of the contexts we wish to support.
pub fn __log_message<S: AsRef<str>, Tz: TimeZone, L: Logger + Logger<ReturnType = T>, T>(
    log_level: LogLevel,
    message: S,
    format: Format<Tz>,
    logger: &mut L,
) -> Option<T>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Local>: From<DateTime<Tz>>,
    DateTime<Utc>: From<DateTime<Tz>>,
    DateTime<Tz>: Copy,
{
    let item = LogItem::new(format, log_level, message.as_ref());

    if logger.can_log_item(&item) {
        return Some(logger.log_item(item));
    } else {
        return None;
    }
}

#[cfg(test)]
mod tests {
    use crate::logger::StringLogger;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_error_macro() {
        let mut logger = StringLogger::new_tz();
        logger.set_override(
            default_format_custom_tz!(Utc).set_constant_time(DateTime::from(
                DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
            )),
        );
        let content = error!("my message", logger).unwrap();

        assert_eq!(
            content,
            format!(
                "[10:52:37] ({} {}:13) Error: my message",
                module_path!(),
                line!() - 11
            )
        );
    }

    #[test]
    fn test_warning_macro() {
        let mut logger = StringLogger::new_tz();
        logger.set_override(
            default_format_custom_tz!(Utc).set_constant_time(DateTime::from(
                DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
            )),
        );
        let content = warning!("my message", logger).unwrap();

        assert_eq!(
            content,
            format!(
                "[10:52:37] ({} {}:13) Warning: my message",
                module_path!(),
                line!() - 11
            )
        );
    }

    #[test]
    fn test_state_change_macro() {
        let mut logger = StringLogger::new_tz();
        logger.set_override(
            default_format_custom_tz!(Utc).set_constant_time(DateTime::from(
                DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
            )),
        );
        let content = state_change!("my message", logger).unwrap();

        assert_eq!(
            content,
            format!(
                "[10:52:37] ({} {}:13) StateChange: my message",
                module_path!(),
                line!() - 11,
            )
        );
    }

    #[test]
    fn test_info_macro() {
        let mut logger = StringLogger::new_tz();
        logger.set_override(
            default_format_custom_tz!(Utc).set_constant_time(DateTime::from(
                DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
            )),
        );
        let content = info!("my message", logger).unwrap();

        assert_eq!(
            content,
            format!(
                "[10:52:37] ({} {}:13) Information: my message",
                module_path!(),
                line!() - 11,
            )
        );
    }
}

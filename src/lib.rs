//! A basic logging crate used in the muxide terminal multiplexer.
//!
//! The `muxide_logging` crate is built similar to the [Log](https://crates.io/crates/log) crate
//! however it was designed from scratch, primarily for interest's sake but also because I wanted
//! an interface I better understood and catered to my specific needs.
//!
//! Whilst most structs in this crate support custom timezones, it is not considered a principle
//! focus and the support is mainly untested but possible if desired. The main intention is to use
//! the [Local](chrono::Local) timezone where possible and by default.

pub mod format;
pub mod log;
pub mod logger;
#[macro_use]
mod macros;

// Internal undocumented methods used within the macros.
pub use macros::{__default_log_message, __log_message};

pub(crate) type DefaultLogger = FileLogger<chrono::Local>;
use crate::log::LogLevel;
use lazy_static::lazy_static;
use logger::FileLogger;
use std::path::Path;
use std::sync::Mutex;

lazy_static! {
    /// The default logger. It is thread-safe and utilised by default in all the macros unless
    /// otherwise specified..
    ///
    /// Utilise the [set_output_file] and [close_output_file] to open and close respectively, the file
    /// used for logging.
    pub static ref DEFAULT_LOGGER: Mutex<DefaultLogger> = Mutex::new(DefaultLogger::new());
}

/// Set the default logger's output file.
/// Opens a new file to append new logs to. This method WILL block if another process is currently
/// using the default logger.
pub fn set_output_file<P: AsRef<Path>>(path: P) -> Result<(), String> {
    return DEFAULT_LOGGER
        .lock()
        .map_err(|e| e.to_string())?
        .open_file(path)
        .map_err(|e| e.to_string());
}

/// Close the file opened by the default logger. This method WILL block if another process is
/// currently using the default logger.
pub fn close_output_file() -> Result<(), String> {
    DEFAULT_LOGGER
        .lock()
        .map_err(|e| e.to_string())?
        .close_file();

    return Ok(());
}

/// Prevents the default logger from logging logs at the specified levels.
pub fn restrict_log_levels(prohibited_levels: &[LogLevel]) -> Result<(), String> {
    DEFAULT_LOGGER
        .lock()
        .map_err(|e| e.to_string())?
        .restrict_log_levels(prohibited_levels);

    return Ok(());
}

/// Removes any restrictions on the specified restricted log levels. The [restrict_log_levels] method
/// must have been called on the specified log levels for any changes to have an affect.
pub fn allow_log_levels(allowed_log_levels: &[LogLevel]) -> Result<(), String> {
    DEFAULT_LOGGER
        .lock()
        .map_err(|e| e.to_string())?
        .allow_log_levels(allowed_log_levels);

    return Ok(());
}

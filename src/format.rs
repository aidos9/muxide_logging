//! Formatting for logging messages

use crate::log::LogLevel;
use chrono::{DateTime, Local, TimeZone, Utc};
use std::ops::{Index, IndexMut};

#[derive(Clone, PartialEq, Debug)]
/// A possible item type for used to dictate the format of a logged message.
pub enum FormatItem {
    /// The line number of where the log macro was called.
    LineNumber,
    /// The column number of where the log macro was called.
    ColumnNumber,
    /// The module of where the log macro was called.
    ModulePath,
    /// The File of where the log macro was called.
    File,
    /// The log level of the log.
    LogLevel,
    /// The message used to log.
    LogString,
    /// Display the time with a specified format dictated in [chrono](https://docs.rs/chrono/).
    TimeString(String),
    /// A custom character.
    CustomCharacter(char),
    ///  A custom string.
    CustomString(String),
}

#[derive(Clone, Debug)]
/// This struct dictates the Format of log message. It is used in the macros and is assigned details
/// such as a log messages, line, column, module and file. It can also be used largely, for testing
/// purposes to override whatever time was specified with a custom one.
///
/// The default format is `[HH:MM:SS] (module_path line:column) log_level: log_message` and is
/// created using the [Default] trait.
///
/// # Example
/// The default format.
/// ```
/// use chrono::DateTime;
/// use muxide_logging::format::Format;
/// use muxide_logging::log::LogLevel;
///
/// let mut fmt = Format::default()
///      .set_column(123)
///      .set_line(456)
///      .set_module_path("my_crate::file")
///      .set_constant_time(DateTime::from(
///         DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap(),
///      ));
///
/// assert_eq!(
///     fmt.build_string(LogLevel::Error, "My Error"),
///     "[20:52:37] (my_crate::file 456:123) Error: My Error".to_string()
/// );
///
/// ```
pub struct Format<Tz: TimeZone>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: std::cmp::PartialEq,
    DateTime<Tz>: Copy,
{
    items: Vec<FormatItem>,
    column: Option<usize>,
    line: Option<usize>,
    file: Option<String>,
    module_path: Option<String>,
    custom_time: Option<DateTime<Tz>>,
}

impl Format<Local> {
    /// Create a new empty [Format]
    pub fn new() -> Self {
        return Self {
            items: Vec::new(),
            column: None,
            line: None,
            file: None,
            module_path: None,
            custom_time: None,
        };
    }
}

impl<Tz: TimeZone> Format<Tz>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy,
{
    /// Create a new empty [Format] but for non-local timezones.
    pub fn new_tz() -> Self {
        return Self {
            items: Vec::new(),
            column: None,
            line: None,
            file: None,
            module_path: None,
            custom_time: None,
        };
    }

    /// Same as [default](Format::default) but with support for non-local timezones.
    pub fn default_tz() -> Self {
        return crate::build_format_from_items_tz!(
            FormatItem::CustomCharacter('['),
            FormatItem::TimeString("%k:%M:%S".to_string()),
            FormatItem::CustomString("] (".to_string()),
            FormatItem::ModulePath,
            FormatItem::CustomCharacter(' '),
            FormatItem::LineNumber,
            FormatItem::CustomCharacter(':'),
            FormatItem::ColumnNumber,
            FormatItem::CustomString(") ".to_string()),
            FormatItem::LogLevel,
            FormatItem::CustomString(": ".to_string()),
            FormatItem::LogString
        );
    }

    /// Create a new empty [Format] but with a custom constant time.
    pub fn new_with_constant_time(constant_time: DateTime<Tz>) -> Self {
        return Self {
            items: vec![],
            column: None,
            line: None,
            file: None,
            module_path: None,
            custom_time: Some(constant_time),
        };
    }

    /// Merges two 'Formats' into one, prioritising having a value over not having a value,
    /// but where both contain a value preferring the values from 'a'.
    pub fn merged<T1: TimeZone>(a: &Format<Tz>, b: &Format<T1>) -> Format<Tz>
    where
        T1::Offset: std::fmt::Display,
        DateTime<T1>: Copy,
        DateTime<T1>: Into<DateTime<Tz>>,
    {
        let items = if a.items.len() == 0 {
            b.items.clone()
        } else {
            a.items.clone()
        };

        let column = if a.column.is_none() {
            b.column
        } else {
            a.column
        };

        let line = if a.line.is_none() { b.line } else { a.line };

        let file = if a.file.is_none() {
            b.file.clone()
        } else {
            a.file.clone()
        };

        let module_path = if a.module_path.is_none() {
            b.module_path.clone()
        } else {
            a.module_path.clone()
        };

        let custom_time: Option<DateTime<Tz>> = if a.custom_time.is_none() {
            b.custom_time.as_ref().map(|t| t.clone().into())
        } else {
            a.custom_time.clone()
        };

        return Format {
            items,
            column,
            line,
            file,
            module_path,
            custom_time,
        };
    }

    /// Consumes the format object and builds the formatted output from the log level and log
    /// message.
    pub fn build_string(self, log_level: LogLevel, log_message: &str) -> String {
        let mut item_strings = Vec::with_capacity(self.items.len());

        for item in self.items {
            let string = match item {
                FormatItem::LineNumber => {
                    if self.line.is_some() {
                        self.line.unwrap().to_string()
                    } else {
                        String::new()
                    }
                }
                FormatItem::ColumnNumber => {
                    if self.column.is_some() {
                        self.column.unwrap().to_string()
                    } else {
                        String::new()
                    }
                }
                FormatItem::ModulePath => self
                    .module_path
                    .as_ref()
                    .map(|s| s.clone())
                    .unwrap_or(String::new()),
                FormatItem::LogLevel => log_level.to_string(),
                FormatItem::LogString => log_message.to_string(),
                FormatItem::TimeString(fmt_string) => {
                    if self.custom_time.is_some() {
                        self.custom_time.unwrap().format(&fmt_string).to_string()
                    } else {
                        Local::now().format(&fmt_string).to_string()
                    }
                }
                FormatItem::CustomCharacter(ch) => ch.to_string(),
                FormatItem::CustomString(s) => s,
                FormatItem::File => self
                    .file
                    .as_ref()
                    .map(|s| s.clone())
                    .unwrap_or(String::new()),
            };

            item_strings.push(string);
        }

        return item_strings.join("");
    }

    /// Set the column where the log originated.
    pub fn set_column(mut self, col: usize) -> Self {
        self.column = Some(col);

        return self;
    }

    /// Get the column where the log originated.
    pub fn column(&self) -> Option<usize> {
        return self.column;
    }

    /// Set the line where the log originated.
    pub fn set_line(mut self, line: usize) -> Self {
        self.line = Some(line);

        return self;
    }

    /// Get the line where the log originated.
    pub fn line(&self) -> Option<usize> {
        return self.line;
    }

    /// Set the file where the log originated.
    pub fn set_file(mut self, file: &str) -> Self {
        self.file = Some(file.to_string());

        return self;
    }

    /// Get the file where the log originated.
    pub fn file(&self) -> &Option<String> {
        return &self.file;
    }

    /// Set the module where the log originated.
    pub fn set_module_path(mut self, path: &str) -> Self {
        self.module_path = Some(path.to_string());

        return self;
    }

    /// Get the module where the log originated.
    pub fn module_path(&self) -> &Option<String> {
        return &self.module_path;
    }

    /// Set a custom time to override the current time.
    pub fn set_constant_time(mut self, time: DateTime<Tz>) -> Self {
        self.custom_time = Some(time);

        return self;
    }

    /// Remove the override time.
    pub fn clear_constant_time(mut self) -> Self {
        self.custom_time = None;

        return self;
    }

    /// Append a [FormatItem] to the current sequence.
    pub fn append(mut self, item: FormatItem) -> Self {
        self.items.push(item);

        return self;
    }

    /// Remove the last [FormatItem] from the sequence.
    pub fn pop_last(mut self) -> Self {
        let _ = self.items.pop();

        return self;
    }
}

impl<Tz: TimeZone> Index<usize> for Format<Tz>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy,
{
    type Output = FormatItem;

    fn index(&self, index: usize) -> &Self::Output {
        return self.items.index(index);
    }
}

impl<Tz: TimeZone> IndexMut<usize> for Format<Tz>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        return self.items.index_mut(index);
    }
}

impl<Tz: TimeZone> PartialEq for Format<Tz>
where
    Tz::Offset: std::fmt::Display,
    DateTime<Tz>: Copy + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        return self.file == other.file
            && self.custom_time == other.custom_time
            && self.module_path == other.module_path
            && self.column == other.column
            && self.line == other.line
            && self.items == other.items;
    }
}

impl Default for Format<Local> {
    /// Creates a new instance of [Format] with the format
    /// `[HH:MM:SS] (module_path line:column) log_level: log_message`
    fn default() -> Self {
        return crate::build_format_from_items!(
            FormatItem::CustomCharacter('['),
            FormatItem::TimeString("%k:%M:%S".to_string()),
            FormatItem::CustomString("] (".to_string()),
            FormatItem::ModulePath,
            FormatItem::CustomCharacter(' '),
            FormatItem::LineNumber,
            FormatItem::CustomCharacter(':'),
            FormatItem::ColumnNumber,
            FormatItem::CustomString(") ".to_string()),
            FormatItem::LogLevel,
            FormatItem::CustomString(": ".to_string()),
            FormatItem::LogString
        );
    }
}

impl From<Format<Local>> for Format<Utc> {
    fn from(fmt: Format<Local>) -> Self {
        return Self {
            items: fmt.items,
            column: fmt.column,
            line: fmt.line,
            file: fmt.file,
            module_path: fmt.module_path,
            custom_time: fmt.custom_time.map(|dt| dt.into()),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::format::{Format, FormatItem};
    use crate::log::LogLevel;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_default() {
        assert_eq!(
            Format::default(),
            Format {
                items: vec![
                    FormatItem::CustomCharacter('['),
                    FormatItem::TimeString("%k:%M:%S".to_string()),
                    FormatItem::CustomString("] (".to_string()),
                    FormatItem::ModulePath,
                    FormatItem::CustomCharacter(' '),
                    FormatItem::LineNumber,
                    FormatItem::CustomCharacter(':'),
                    FormatItem::ColumnNumber,
                    FormatItem::CustomString(") ".to_string()),
                    FormatItem::LogLevel,
                    FormatItem::CustomString(": ".to_string()),
                    FormatItem::LogString
                ],
                column: None,
                line: None,
                file: None,
                module_path: None,
                custom_time: None
            }
        )
    }

    #[test]
    fn test_build_default() {
        // Use Utc for these tests so they pass on other machines
        assert_eq!(
            Format::<Utc>::default_tz()
                .set_column(0)
                .set_line(123)
                .set_module_path("muxide_logger::log")
                .set_constant_time(DateTime::from(
                    DateTime::parse_from_rfc2822("Tue, 1 Jul 2003 10:52:37 +0000").unwrap()
                ))
                .build_string(LogLevel::Warning, "Some Warning"),
            "[10:52:37] (muxide_logger::log 123:0) Warning: Some Warning".to_string(),
        )
    }

    #[test]
    fn test_format_index() {
        assert_eq!(
            Format::new()
                .append(FormatItem::LogLevel)
                .append(FormatItem::ColumnNumber)[0],
            FormatItem::LogLevel
        );
    }
}

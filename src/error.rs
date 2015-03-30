use std::fmt::{Display, Formatter, Error};

/// Error returned when the passed Json did not have the required structure.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidJsonError { message: String }

impl InvalidJsonError {
    /// Create a new error instance.
    pub fn new(message: &str) -> InvalidJsonError {
        InvalidJsonError { message: String::from_str(message) }
    }
}

impl Display for InvalidJsonError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        self.message.fmt(formatter)
    }
}

/// Error returned when a string could not be parsed as an `EventType`.
///
/// This may occur if an event is provided by DaZeus which is unknown by this implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseEventTypeError { _priv: () }

impl ParseEventTypeError {
    /// Create a new error instance.
    pub fn new() -> ParseEventTypeError {
        ParseEventTypeError { _priv: () }
    }
}

/// Error returned when a string could not be parsed as a `ConfigGroup`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseConfigGroupError { _priv: () }

impl ParseConfigGroupError {
    /// Create a new error instance.
    pub fn new() -> ParseConfigGroupError {
        ParseConfigGroupError { _priv: () }
    }
}

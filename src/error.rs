use std::io::Error as IoError;
use crate::serialize::json::ParserError as JsonParserError;
use std::str::Utf8Error;
use std::fmt::{Display, Formatter};
use std::fmt::Error as FmtError;

/// Error returned when the passed Json did not have the required structure.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidJsonError { message: String }

impl InvalidJsonError {
    /// Create a new error instance.
    pub fn new(message: &str) -> InvalidJsonError {
        InvalidJsonError { message: message.to_string() }
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

/// Error when an unexpected or invalid response was received from DaZeus
#[derive(Debug, Clone, PartialEq)]
pub struct ReceiveError { _priv: () }

impl ReceiveError {
    pub fn new() -> ReceiveError {
        ReceiveError { _priv: () }
    }
}

#[derive(Debug)]
pub enum Error {
    JsonParserError(JsonParserError),
    IoError(IoError),
    Utf8Error(Utf8Error),
    InvalidJsonError(InvalidJsonError),
    ReceiveError(ReceiveError),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::IoError(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

impl From<JsonParserError> for Error {
    fn from(err: JsonParserError) -> Error {
        Error::JsonParserError(err)
    }
}

impl From<InvalidJsonError> for Error {
    fn from(err: InvalidJsonError) -> Error {
        Error::InvalidJsonError(err)
    }
}

impl From<ReceiveError> for Error {
    fn from(err: ReceiveError) -> Error {
        Error::ReceiveError(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{:?}", self)
    }
}

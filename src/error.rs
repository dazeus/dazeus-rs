use std::fmt::{Display, Formatter, Error};

#[derive(Debug, Clone, PartialEq)]
pub struct InvalidJsonError { message: String }

impl InvalidJsonError {
    pub fn new(message: &str) -> InvalidJsonError {
        InvalidJsonError { message: String::from_str(message) }
    }
}

impl Display for InvalidJsonError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        self.message.fmt(formatter)
    }
}

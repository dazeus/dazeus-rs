#[derive(Debug, Clone, PartialEq)]
pub struct InvalidJsonError { message: String }

impl InvalidJsonError {
    pub fn new(message: &str) -> InvalidJsonError {
        InvalidJsonError { message: String::from_str(message) }
    }
}

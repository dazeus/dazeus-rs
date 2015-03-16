use serialize::json::Json;
use super::error::InvalidJsonError;

/// The response from a command send to the DaZeus server
#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    data: Json
}

impl Response {
    pub fn from_json(data: Json) -> Result<Response, InvalidJsonError> {
        Ok(Response { data: data })
    }
}

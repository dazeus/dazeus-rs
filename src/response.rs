use super::error::InvalidJsonError;

use serde_json::Map;
use serde_json::Value as JsonValue;

/// The response from a command send to the DaZeus server.
#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    data: JsonValue,
}

impl Response {
    /// Create a new response based upon a failure message.
    ///
    /// This is used where the API expected a response returned but the DaZeus core could not
    /// provide a valid response.
    pub fn for_fail(msg: &str) -> Response {
        let mut obj = Map::new();
        obj.insert("success".to_string(), JsonValue::Bool(false));
        obj.insert("reason".to_string(), JsonValue::String(msg.to_string()));

        Response {
            data: JsonValue::Object(obj),
        }
    }

    /// Create a new response based upon a successful operation.
    ///
    /// This is used when the API expected a response, but the DaZeus core was not called.
    pub fn for_success() -> Response {
        let mut obj = Map::new();
        obj.insert("success".to_string(), JsonValue::Bool(true));

        Response {
            data: JsonValue::Object(obj),
        }
    }

    /// Create a new response based on a Json object.
    ///
    /// This is used by the bindings to create a new Response based on a json blob returned by the
    /// DaZeus core instance.
    pub fn from_json(data: &JsonValue) -> Result<Response, InvalidJsonError> {
        Ok(Response { data: data.clone() })
    }

    /// Retrieve a property from the data object or return a default if it doesn't exist.
    pub fn get_or<'a>(&'a self, prop: &'a str, default: &'a JsonValue) -> &'a JsonValue {
        match self.get(prop) {
            Some(val) => val,
            None => default,
        }
    }

    /// Retrieve a property from the data object.
    ///
    /// Returns `Some(data)` if the property exists, or `None` if the property doesn't exist.
    pub fn get<'a>(&'a self, prop: &'a str) -> Option<&'a JsonValue> {
        match self.data {
            JsonValue::Object(ref obj) => obj.get(prop),
            _ => None,
        }
    }

    /// Retrieve a string from the data object.
    ///
    /// Returns `Some(str)` if the property exists and it was a string property, or `None` if the
    /// property doesn't exist, or if it isn't of type `Json::String`.
    pub fn get_str<'a>(&'a self, prop: &'a str) -> Option<&'a str> {
        match self.get(prop) {
            Some(&JsonValue::String(ref s)) => Some(&s[..]),
            _ => None,
        }
    }

    /// Retrieve a string from the data object, or return a default if no such string can be found.
    pub fn get_str_or<'a>(&'a self, prop: &'a str, default: &'a str) -> &'a str {
        match self.get_str(prop) {
            Some(s) => s,
            None => default,
        }
    }

    /// Returns whether or not a property with the given name exists.
    pub fn has(&self, prop: &str) -> bool {
        self.get_str(prop).is_some()
    }

    /// Check whether a Response contains a `success` property and whether it was true.
    pub fn has_success(&self) -> bool {
        match self.get("success") {
            Some(&JsonValue::Bool(true)) => true,
            _ => false,
        }
    }
}

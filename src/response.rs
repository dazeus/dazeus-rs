use serialize::json::Json;
use serialize::json::Object;
use super::error::InvalidJsonError;

/// The response from a command send to the DaZeus server
#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    data: Json
}

impl Response {
    pub fn from_fail(msg: &str) -> Response {
        let mut obj = Object::new();
        obj.insert("success".to_string(), Json::Boolean(false));
        obj.insert("reason".to_string(), Json::String(String::from_str(msg)));

        Response { data: Json::Object(obj) }
    }

    pub fn for_success() -> Response {
        let mut obj = Object::new();
        obj.insert("success".to_string(), Json::Boolean(true));

        Response { data: Json::Object(obj) }
    }

    pub fn from_json(data: Json) -> Result<Response, InvalidJsonError> {
        Ok(Response { data: data })
    }

    pub fn get_or<'a>(&'a self, prop: &'a str, default: &'a Json) -> &'a Json {
        match self.get(prop) {
            Some(val) => val,
            None => default,
        }
    }

    pub fn get<'a>(&'a self, prop: &'a str) -> Option<&'a Json> {
        match self.data {
            Json::Object(ref obj) => {
                obj.get(prop)
            },
            _ => None,
        }
    }

    pub fn get_str<'a>(&'a self, prop: &'a str) -> Option<&'a str> {
        match self.get(prop) {
            Some(&Json::String(ref s)) => Some(&s[..]),
            _ => None,
        }
    }

    pub fn get_str_or<'a>(&'a self, prop: &'a str, default: &'a str) -> &'a str {
        match self.get_str(prop) {
            Some(s) => s,
            None => default,
        }
    }

    pub fn has_success(&self) -> bool {
        match self.get("success") {
            Some(&Json::Boolean(true)) => true,
            _ => false,
        }
    }
}

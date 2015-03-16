use serialize::json::{ToJson, Json, Array};

/// A scope for retrieving permissions and properties
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub network: Option<String>,
    pub sender: Option<String>,
    pub receiver: Option<String>,
}

impl Scope {
    /// Construct a new scope with the specified limitations for network, sender and receiver
    pub fn new(network: Option<String>, sender: Option<String>, receiver: Option<String>) -> Scope {
        Scope { network: network, sender: sender, receiver: receiver }
    }

    /// Scope to everyone and anything
    pub fn any() -> Scope {
        Scope::new(None, None, None)
    }

    /// Scope to a specific network
    pub fn network(network: &str) -> Scope {
        Scope::new(Some(String::from_str(network)), None, None)
    }

    /// Scope to a specific sender (typically a channel)
    pub fn sender(network: &str, sender: &str) -> Scope {
        Scope::new(Some(String::from_str(network)), Some(String::from_str(sender)), None)
    }

    /// Scope to a specific receiver (typically a user)
    pub fn receiver(network: &str, receiver: &str) -> Scope {
        Scope::new(Some(String::from_str(network)), None, Some(String::from_str(receiver)))
    }

    /// Scope to a specific receiver and channel (typically a user in a channel)
    pub fn to(network: &str, sender: &str, receiver: &str) -> Scope {
        Scope::new(Some(String::from_str(network)), Some(String::from_str(sender)), Some(String::from_str(receiver)))
    }

    pub fn is_any(&self) -> bool {
        self.network == None && self.sender == None && self.receiver == None
    }
}

impl ToJson for Scope {
    fn to_json(&self) -> Json {
        let mut arr = Array::new();
        arr.push(match self.network {
            None => Json::Null,
            Some(ref s) => Json::String(s.clone()),
        });
        arr.push(match self.sender {
            None => Json::Null,
            Some(ref s) => Json::String(s.clone()),
        });
        arr.push(match self.receiver {
            None => Json::Null,
            Some(ref s) => Json::String(s.clone()),
        });
        Json::Array(arr)
    }
}

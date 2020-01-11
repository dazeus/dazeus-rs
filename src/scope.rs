use serde_json::Value as JsonValue;

/// A scope for retrieving permissions and properties.
///
/// A scope is an optional limitation on which some property or permission applies. A scope
/// consists of three different elements:
///
/// * The `network` indicates for which network some property or permission should be stored.
/// * The `sender` indicates the IRC user for which some property or permission should be stored.
/// * The `receiver` indicates the channel for which some property or permission should be stored.
///
/// Note that for a sender or receiver scope, you also should provide a network, as it makes no
/// sense to for example provide a permission to the same channel on different networks (they are
/// only the same in name, but might be completely different in context).
///
/// The most generic scope (and easiest one to start with) is one applied to everything. Such a
/// scope can be created by the `Scope::any()` method.
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    /// The network on which the scope is limited (if any).
    pub network: Option<String>,

    /// The sender on which the scope is limited (if any).
    pub sender: Option<String>,

    /// The receiver on which the scope is limited (if any).
    pub receiver: Option<String>,
}

impl Scope {
    /// Construct a new scope with the specified limitations for network, sender and receiver
    pub fn new(network: Option<String>, sender: Option<String>, receiver: Option<String>) -> Scope {
        Scope {
            network,
            sender,
            receiver,
        }
    }

    /// Scope to everyone and anything
    pub fn any() -> Scope {
        Scope::new(None, None, None)
    }

    /// Scope to a specific network
    pub fn network(network: &str) -> Scope {
        Scope::new(Some(network.to_string()), None, None)
    }

    /// Scope to a specific sender (typically a channel)
    pub fn sender(network: &str, sender: &str) -> Scope {
        Scope::new(Some(network.to_string()), Some(sender.to_string()), None)
    }

    /// Scope to a specific receiver (typically a user)
    pub fn receiver(network: &str, receiver: &str) -> Scope {
        Scope::new(Some(network.to_string()), None, Some(receiver.to_string()))
    }

    /// Scope to a specific receiver and channel (typically a user in a channel)
    pub fn to(network: &str, sender: &str, receiver: &str) -> Scope {
        Scope::new(
            Some(network.to_string()),
            Some(sender.to_string()),
            Some(receiver.to_string()),
        )
    }

    /// Checks whether the scope is set to be applied to everything.
    pub fn is_any(&self) -> bool {
        self.network == None && self.sender == None && self.receiver == None
    }
}

impl Scope {
    pub fn to_json(&self) -> JsonValue {
        let mut arr = Vec::new();
        if self.network.is_some() || self.sender.is_some() || self.receiver.is_some() {
            arr.push(match self.network {
                None => JsonValue::Null,
                Some(ref s) => JsonValue::String(s.clone()),
            });

            if self.sender.is_some() || self.receiver.is_some() {
                arr.push(match self.sender {
                    None => JsonValue::Null,
                    Some(ref s) => JsonValue::String(s.clone()),
                });

                if let Some(ref s) = self.receiver {
                    arr.push(JsonValue::String(s.clone()));
                }
            }
        }
        JsonValue::Array(arr)
    }
}

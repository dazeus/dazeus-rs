/// A scope for retrieving permissions and properties
pub struct Scope<'a> {
    pub network: Option<&'a str>,
    pub sender: Option<&'a str>,
    pub receiver: Option<&'a str>,
}

impl<'a> Scope<'a> {
    /// Construct a new scope with the specified limitations for network, sender and receiver
    pub fn new(network: Option<&'a str>, sender: Option<&'a str>, receiver: Option<&'a str>) -> Scope<'a> {
        Scope { network: network, sender: sender, receiver: receiver }
    }

    /// Scope to everyone and anything
    pub fn any() -> Scope<'a> {
        Scope::new(None, None, None)
    }

    /// Scope to a specific network
    pub fn network(network: &'a str) -> Scope<'a> {
        Scope::new(Some(network), None, None)
    }

    /// Scope to a specific sender (typically a channel)
    pub fn sender(network: &'a str, sender: &'a str) -> Scope<'a> {
        Scope::new(Some(network), Some(sender), None)
    }

    /// Scope to a specific receiver (typically a user)
    pub fn receiver(network: &'a str, receiver: &'a str) -> Scope<'a> {
        Scope::new(Some(network), None, Some(receiver))
    }

    /// Scope to a specific receiver and channel (typically a user in a channel)
    pub fn to(network: &'a str, sender: &'a str, receiver: &'a str) -> Scope<'a> {
        Scope::new(Some(network), Some(sender), Some(receiver))
    }
}

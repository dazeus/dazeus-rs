use super::scope::Scope;
use serialize::json::{Json, ToJson, Object, Array};


const PROTOCOL_VERSION: &'static str = "1";


#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    Subscribe(String),
    Unsubscribe(String),
    SubscribeCommand(String, Option<String>),
    Networks,
    Channels(String),
    Message(String, String, String),
    Notice(String, String, String),
    Ctcp(String, String, String),
    CtcpReply(String, String, String),
    Action(String, String, String),
    Names(String, String),
    Whois(String, String),
    Join(String, String),
    Part(String, String),
    Nick(String),
    Handshake(String, String, Option<String>),
    Config(String, Option<String>),
    GetProperty(String, Scope),
    SetProperty(String, String, Scope),
    UnsetProperty(String, Scope),
    PropertyKeys(String, Scope),
    SetPermission(String, bool, Scope),
    HasPermission(String, bool, Scope),
    UnsetPermission(String, Scope),
}

impl Request {
    fn get_json_name(&self) -> Json {
        let s = match *self {
            Request::Subscribe(_) => "subscribe",
            Request::Unsubscribe(_) => "unsubscribe",
            Request::SubscribeCommand(_, _) => "command",
            Request::Networks => "networks",
            Request::Channels(_) => "channels",
            Request::Message(_, _, _) => "message",
            Request::Notice(_, _, _) => "notice",
            Request::Ctcp(_, _, _) => "ctcp",
            Request::CtcpReply(_, _, _) => "ctcp_rep",
            Request::Action(_, _, _) => "action",
            Request::Names(_, _) => "names",
            Request::Whois(_, _) => "whois",
            Request::Join(_, _) => "join",
            Request::Part(_, _) => "part",
            Request::Nick(_) => "nick",
            Request::Handshake(_, _, _) => "handshake",
            Request::Config(_, _) => "config",
            Request::GetProperty(_, _) => "property",
            Request::SetProperty(_, _, _) => "property",
            Request::UnsetProperty(_, _) => "property",
            Request::PropertyKeys(_, _) => "property",
            Request::SetPermission(_, _, _) => "permission",
            Request::HasPermission(_, _, _) => "permission",
            Request::UnsetPermission(_, _) => "permission",
        };

        Json::String(String::from_str(s))
    }

    fn get_action_type(&self) -> String {
        let s = match *self {
            Request::Networks | Request::Channels(_) | Request::Nick(_) | Request::Config(_, _) => "get",
            _ => "do",
        };

        String::from_str(s)
    }
}

impl ToJson for Request {
    fn to_json(&self) -> Json {
        let mut obj = Object::new();
        obj.insert(self.get_action_type(), self.get_json_name());

        let mut params = Array::new();

        macro_rules! push_str { ($x: expr) => ( params.push(Json::String($x.clone())) ) }

        match *self {
            Request::Subscribe(ref evt) => {
                push_str!(evt);
            },
            Request::Unsubscribe(ref evt) => {
                push_str!(evt);
            },
            Request::SubscribeCommand(ref cmd, Some(ref network)) => {
                push_str!(cmd);
                push_str!(network);
            },
            Request::SubscribeCommand(ref cmd, None) => {
                push_str!(cmd);
            },
            Request::Networks => (),
            Request::Channels(ref network)
            | Request::Nick(ref network) => {
                push_str!(network);
            },
            Request::Message(ref network, ref channel, ref message)
            | Request::Notice(ref network, ref channel, ref message)
            | Request::Ctcp(ref network, ref channel, ref message)
            | Request::CtcpReply(ref network, ref channel, ref message)
            | Request::Action(ref network, ref channel, ref message) => {
                push_str!(network);
                push_str!(channel);
                push_str!(message);
            },
            Request::Names(ref network, ref channel)
            | Request::Join(ref network, ref channel)
            | Request::Part(ref network, ref channel) => {
                push_str!(network);
                push_str!(channel);
            },
            Request::Whois(ref network, ref user) => {
                push_str!(network);
                push_str!(user);
            },
            Request::Handshake(ref name, ref version, Some(ref config_name)) => {
                push_str!(name);
                push_str!(version);
                push_str!(String::from_str(PROTOCOL_VERSION));
                push_str!(config_name);
            },
            Request::Handshake(ref name, ref version, None) => {
                push_str!(name);
                push_str!(version);
                push_str!(String::from_str(PROTOCOL_VERSION));
                push_str!(name);
            },
            Request::Config(ref key, Some(ref group)) => {
                push_str!(key);
                push_str!(group);
            },
            Request::Config(ref key, None) => {
                push_str!(key);
                push_str!(String::from_str("plugin"));
            }
            Request::GetProperty(ref property, ref scope) => {
                push_str!(String::from_str("get"));
                push_str!(property);
                if !scope.is_any() {
                    obj.insert(String::from_str("scope"), scope.to_json());
                }
            },
            Request::SetProperty(ref property, ref value, ref scope) => {
                push_str!(String::from_str("set"));
                push_str!(property);
                push_str!(value);
                if !scope.is_any() {
                    obj.insert(String::from_str("scope"), scope.to_json());
                }
            },
            Request::UnsetProperty(ref property, ref scope) => {
                push_str!(String::from_str("unset"));
                push_str!(property);
                if !scope.is_any() {
                    obj.insert(String::from_str("scope"), scope.to_json());
                }
            },
            Request::PropertyKeys(ref prefix, ref scope) => {
                push_str!(String::from_str("keys"));
                push_str!(prefix);
                if !scope.is_any() {
                    obj.insert(String::from_str("scope"), scope.to_json());
                }
            },
            Request::SetPermission(ref permission, ref default, ref scope) => {
                push_str!(String::from_str("set"));
                push_str!(permission);
                params.push(Json::Boolean(*default));
                if !scope.is_any() {
                    obj.insert(String::from_str("scope"), scope.to_json());
                }
            },
            Request::HasPermission(ref permission, ref default, ref scope) => {
                push_str!(String::from_str("get"));
                push_str!(permission);
                params.push(Json::Boolean(*default));
                if !scope.is_any() {
                    obj.insert(String::from_str("scope"), scope.to_json());
                }
            },
            Request::UnsetPermission(ref permission, ref scope) => {
                push_str!(String::from_str("unset"));
                push_str!(permission);
                if !scope.is_any() {
                    obj.insert(String::from_str("scope"), scope.to_json());
                }
            },
        }

        if params.len() > 0 {
            obj.insert(String::from_str("params"), Json::Array(params));
        }

        Json::Object(obj)
    }
}

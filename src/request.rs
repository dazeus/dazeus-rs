use super::error::ParseConfigGroupError;
use super::event::EventType;
use super::scope::Scope;
use rustc_serialize::json::{Array, Json, Object, ToJson};
use std::str::FromStr;
use std::string::ToString;

/// The version of the DaZeus plugin communication protocol that these bindings understand.
pub const PROTOCOL_VERSION: &str = "1";

/// A `String` for the target IRC network.
pub type Network = String;

/// A `String` containing the message to be sent.
pub type Message = String;

/// A `String` containing the target (receiver) of some command or message.
///
/// Typically this is a client or some channel on an IRC network.
pub type Target = String;

/// A `String` indicating the name of some command.
pub type Command = String;

/// A `String` indicating the name of the plugin. This is used for retrieving configuration.
pub type PluginName = String;

/// A `String` indicating the version of the protocol used by the bindings.
pub type PluginVersion = String;

/// The type of config that should be retrieved.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigGroup {
    /// Indicates a config value that should be retrieved from the plugin settings.
    Plugin,
    /// Indicates a config value that should be retrieved from the core settings.
    Core,
}

impl ToString for ConfigGroup {
    fn to_string(&self) -> String {
        match *self {
            ConfigGroup::Plugin => "plugin".to_string(),
            ConfigGroup::Core => "core".to_string(),
        }
    }
}

impl FromStr for ConfigGroup {
    type Err = ParseConfigGroupError;

    fn from_str(s: &str) -> Result<Self, ParseConfigGroupError> {
        match &s.to_ascii_lowercase()[..] {
            "plugin" => Ok(ConfigGroup::Plugin),
            "core" => Ok(ConfigGroup::Core),
            _ => Err(ParseConfigGroupError::new()),
        }
    }
}

/// An enum of all requests that can be sent to your DaZeus instance.
///
/// Note that typically you won't create these request instances directly. Instead you can use the
/// different `DaZeus` methods. However if you wish, you can directly use `DaZeus::send()` to send
/// these requests yourself.
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    /// Subscribe to a certain event type.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::Subscribe(EventType::PrivMsg);
    /// ```
    Subscribe(EventType),
    /// Unsubscribe from an event.
    ///
    /// Note that you cannot specify `EventType::Command()` for this request, as registered
    /// commands cannot be unregistered from the DaZeus server. To stop listening, just let DaZeus
    /// remove the listener.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::Unsubscribe(EventType::Join);
    /// ```
    Unsubscribe(EventType),
    /// Subscribe to a command (optionally on a specific network).
    ///
    /// You can use `Request::Subscribe(EventType::Command("example".to_string())` as an
    /// alternative to `Request::SubscribeCommand("example".to_string())`. Note that the
    /// former does not allow you to optionally specify a network on which the command is actively
    /// listened to.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::SubscribeCommand("greet".to_string(), Some("freenode".to_string()));
    /// ```
    SubscribeCommand(Command, Option<Network>),
    /// Retrieve a list of networks that the DaZeus core is currently connected to.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Networks;
    /// ```
    Networks,
    /// Retrieve a list of channels on the specified network that the bot has joined.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Channels("freenode".to_string());
    /// ```
    Channels(Network),
    /// Request to send a message to a specific target on some network.
    ///
    /// This will request DaZeus to send a PRIVMSG.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Message("freenode".to_string(),
    ///                  "#botters-test".to_string(),
    ///                  "Hello!".to_string());
    /// ```
    Message(Network, Target, Message),
    /// Request to send a notice to some target on some network.
    ///
    /// This will request DaZeus to send a NOTICE.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Notice("example".to_string(), "MrExample".to_string(), "Message!".to_string());
    /// ```
    Notice(Network, Target, Message),
    /// Request to send a CTCP message to some client on some network.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Ctcp("example".to_string(), "MrExample".to_string(), "VERSION".to_string());
    /// ```
    Ctcp(Network, Target, Message),
    /// Request to send a CTCP message reply to some client on some network.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::CtcpReply("example".to_string(), "MrExample".to_string(), "VERSION DaZeus 2.0".to_string());
    /// ```
    CtcpReply(Network, Target, Message),
    /// Request to send a CTCP ACTION message to some target on some network.
    ///
    /// A CTCP ACTION is most known by users as the `/me` command.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Action("example".to_string(), "#example".to_string(), "is creating an example".to_string());
    /// ```
    Action(Network, Target, Message),
    /// Request to send the list of names in some channel.
    ///
    /// Note that such a request will generate an `EventType::Names` event if the server allows it,
    /// instead of responding to this request directly.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Names("freenode".to_string(), "#freenode".to_string());
    /// ```
    Names(Network, Target),
    /// Request to send a whois on some target.
    ///
    /// Note that such a request will generate an `EventType::Whois` event if the server allows it,
    /// instead of responding to this request directly.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Whois("example".to_string(), "MrExample".to_string());
    /// ```
    Whois(Network, Target),
    /// Request to join a channel on some network.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Join("freenode".to_string(), "#freenode".to_string());
    /// ```
    Join(Network, Target),
    /// Request to leave a channel on some network.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Part("freenode".to_string(), "#freenode".to_string());
    /// ```
    Part(Network, Target),
    /// Request the nickname of the bot on a network.
    ///
    /// # Example
    /// ```
    /// # use dazeus::Request;
    /// Request::Nick("freenode".to_string());
    /// ```
    Nick(Network),
    /// Request for a handshake to the DaZeus Core.
    ///
    /// The handshake allows the plugin to retrieve configuration options.
    ///
    /// The second parameter of this request variant is the plugin version. Note that this
    /// parameter should be equal to the version that these bindings understand.
    ///
    /// The optional third parameter may provide an alternative name to be used to retrieve
    /// options from the DaZeus core config. By default the name of the plugin will be used.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::Handshake("my_plugin".to_string(), PROTOCOL_VERSION.to_string(), None);
    /// ```
    Handshake(PluginName, PluginVersion, Option<String>),
    /// Retrieve some option from the DaZeus core config.
    ///
    /// The second parameter is the section from which the configuration parameter should be
    /// retrieved. This may either be `ConfigGroup::Core` or `ConfigGroup::Plugin`
    ///
    /// Note that in order to successfully retrieve these configuration values the plugin first
    /// needs to have completed a succesful handshake with the core.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::Config("highlight".to_string(), ConfigGroup::Core);
    /// ```
    Config(String, ConfigGroup),
    /// Retrieve a property from the internal DaZeus core database.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::GetProperty("example".to_string(), Scope::any());
    /// ```
    GetProperty(String, Scope),
    /// Set a property in the internal DaZeus core database.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::SetProperty("example".to_string(), "value".to_string(), Scope::any());
    /// ```
    SetProperty(String, String, Scope),
    /// Remove a property from the internal DaZeus core database.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::UnsetProperty("example".to_string(), Scope::any());
    /// ```
    UnsetProperty(String, Scope),
    /// Retrieve a set of keys that is available for some prefix and scope.
    ///
    /// The first parameter of the variant is the prefix string for retrieving property keys.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::PropertyKeys("path.to".to_string(), Scope::network("example"));
    /// ```
    PropertyKeys(String, Scope),
    /// Set a permission in the permission database of the DaZeus core.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::SetPermission("edit".to_string(), true, Scope::sender("example", "MrExample"));
    /// ```
    SetPermission(String, bool, Scope),
    /// Request a permission to be retrieved from the permission database of the DaZeus core.
    ///
    /// The second parameter of this variant indicates the default value that the core should
    /// return if the permission was not found inside the permission database.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::HasPermission("edit".to_string(), false, Scope::sender("example", "MrExample"));
    /// ```
    HasPermission(String, bool, Scope),
    /// Remove a permission from the permission database of the DaZeus core.
    ///
    /// # Example
    /// ```
    /// # use dazeus::*;
    /// Request::UnsetPermission("edit".to_string(), Scope::sender("example", "MrExample"));
    /// ```
    UnsetPermission(String, Scope),
}

impl Request {
    fn get_json_name(&self) -> Json {
        let s = match *self {
            Request::Subscribe(EventType::Command(_)) => "command",
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

        Json::String(s.to_string())
    }

    fn get_action_type(&self) -> String {
        let s = match *self {
            Request::Networks | Request::Channels(_) | Request::Nick(_) | Request::Config(_, _) => {
                "get"
            }
            _ => "do",
        };

        s.to_string()
    }
}

/// Implements transforming the request to a Json object that is ready to be sent a DaZeus core.
impl ToJson for Request {
    fn to_json(&self) -> Json {
        let mut obj = Object::new();
        obj.insert(self.get_action_type(), self.get_json_name());

        let mut params = Array::new();

        macro_rules! push_str {
            ($x: expr) => {
                params.push(Json::String($x.clone()))
            };
        }

        match *self {
            Request::Subscribe(EventType::Command(ref cmd)) => {
                push_str!(cmd);
            }
            Request::Unsubscribe(EventType::Command(_)) => {
                // we can't actually unsubscribe a command
                panic!("Cannot unsubscribe from command");
            }
            Request::Subscribe(ref evt) => {
                push_str!(evt.to_string());
            }
            Request::Unsubscribe(ref evt) => {
                push_str!(evt.to_string());
            }
            Request::SubscribeCommand(ref cmd, Some(ref network)) => {
                push_str!(cmd);
                push_str!(network);
            }
            Request::SubscribeCommand(ref cmd, None) => {
                push_str!(cmd);
            }
            Request::Networks => (),
            Request::Channels(ref network) | Request::Nick(ref network) => {
                push_str!(network);
            }
            Request::Message(ref network, ref channel, ref message)
            | Request::Notice(ref network, ref channel, ref message)
            | Request::Ctcp(ref network, ref channel, ref message)
            | Request::CtcpReply(ref network, ref channel, ref message)
            | Request::Action(ref network, ref channel, ref message) => {
                push_str!(network);
                push_str!(channel);
                push_str!(message);
            }
            Request::Names(ref network, ref channel)
            | Request::Join(ref network, ref channel)
            | Request::Part(ref network, ref channel) => {
                push_str!(network);
                push_str!(channel);
            }
            Request::Whois(ref network, ref user) => {
                push_str!(network);
                push_str!(user);
            }
            Request::Handshake(ref name, ref version, Some(ref config_name)) => {
                push_str!(name);
                push_str!(version);
                push_str!(PROTOCOL_VERSION.to_string());
                push_str!(config_name);
            }
            Request::Handshake(ref name, ref version, None) => {
                push_str!(name);
                push_str!(version);
                push_str!(PROTOCOL_VERSION.to_string());
                push_str!(name);
            }
            Request::Config(ref key, ref ctype) => {
                push_str!(ctype.to_string());
                push_str!(key);
            }
            Request::GetProperty(ref property, ref scope) => {
                push_str!("get".to_string());
                push_str!(property);
                if !scope.is_any() {
                    obj.insert("scope".to_string(), scope.to_json());
                }
            }
            Request::SetProperty(ref property, ref value, ref scope) => {
                push_str!("set".to_string());
                push_str!(property);
                push_str!(value);
                if !scope.is_any() {
                    obj.insert("scope".to_string(), scope.to_json());
                }
            }
            Request::UnsetProperty(ref property, ref scope) => {
                push_str!("unset".to_string());
                push_str!(property);
                if !scope.is_any() {
                    obj.insert("scope".to_string(), scope.to_json());
                }
            }
            Request::PropertyKeys(ref prefix, ref scope) => {
                push_str!("keys".to_string());
                push_str!(prefix);
                if !scope.is_any() {
                    obj.insert("scope".to_string(), scope.to_json());
                }
            }
            Request::SetPermission(ref permission, ref default, ref scope) => {
                push_str!("set".to_string());
                push_str!(permission);
                params.push(Json::Boolean(*default));
                if !scope.is_any() {
                    obj.insert("scope".to_string(), scope.to_json());
                }
            }
            Request::HasPermission(ref permission, ref default, ref scope) => {
                push_str!("get".to_string());
                push_str!(permission);
                params.push(Json::Boolean(*default));
                if !scope.is_any() {
                    obj.insert("scope".to_string(), scope.to_json());
                }
            }
            Request::UnsetPermission(ref permission, ref scope) => {
                push_str!("unset".to_string());
                push_str!(permission);
                if !scope.is_any() {
                    obj.insert("scope".to_string(), scope.to_json());
                }
            }
        }

        if !params.is_empty() {
            obj.insert("params".to_string(), Json::Array(params));
        }

        Json::Object(obj)
    }
}

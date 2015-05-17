use std::str::FromStr;
use serialize::json::Json;
use super::error::{ParseEventTypeError, InvalidJsonError};
use std::ops::Index;
use std::ascii::AsciiExt;

/// The events that could possibly be received from the DaZeus server.
///
/// You can use the variants of this enum to start listening for an event of that type.
/// Every event that you receive will also contain its type.
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    /// A CTCP ACTION event (IRC users will know this as `/me`).
    Action,
    /// A CTCP ACTION sent by the bot from another plugin.
    ActionMe,
    /// A command received by DaZeus.
    ///
    /// Typically a command can be given to the DaZeus server by using a PRIVMSG where
    /// the message is either prefixed by a highlight character or by the name of the bot (in
    /// typical IRC highlight style, eg: `DaZeus: do something`).
    ///
    /// The first word after the highlight is used as the command name. For example when the IRC
    /// user sends a PRIVMSG `DaZeus: start server`, then a `Command("start".to_string())`
    /// variant is sent to the plugin (as long as the plugin has subscribed to such events).
    Command(String),
    /// Signalling that the bot has connected to a new network.
    Connect,
    /// A CTCP event was sent.
    Ctcp,
    /// A CTCP event sent by the bot (from another plugin).
    CtcpMe,
    /// A CTCP_REP event was sent.
    CtcpReply,
    /// Signalling that the bot has disconnected from a network.
    Disconnect,
    /// An invite was sent to the bot.
    Invite,
    /// A JOIN event: an IRC user joined a channel (this may be the bot itself, or another user).
    Join,
    /// A KICK event: an IRC user was kicked from a channel (either the bot itself, or another user).
    Kick,
    /// A MODE event: a mode was changed.
    Mode,
    /// A list of users from some channel (will be sent by the IRC server on request).
    Names,
    /// An event for when the nickname of the bot was changed.
    Nick,
    /// A NOTICE event was sent.
    Notice,
    /// A NUMERIC event was sent (typically contains things such as error messages from the server).
    Numeric,
    /// A PART event: an IRC user left a channel (this may be the bot itself, or another user).
    Part,
    /// An event indicating the response for a ping.
    Pong,
    /// A typical IRC message.
    ///
    /// This is either a user sending a direct message to the bot (indicated by the channel being
    /// equal to the name of the bot), or a message in a channel that was joined by the bot.
    PrivMsg,
    /// A message send by the bot itself via another plugin.
    PrivMsgMe,
    /// A QUIT event: an IRC user disconnects from an IRC server.
    Quit,
    /// A TOPIC event: received when joining a channel or when the topic of a channel is changed.
    Topic,
    /// Unknown event types.
    Unknown,
    /// A WHOIS event: when requested, this is the response to some WHOIS request.
    Whois,
}

impl ToString for EventType {
    fn to_string(&self) -> String {
        match *self {
            EventType::Command(ref s) => format!("COMMAND_{}", s),
            EventType::Action => "ACTION".to_string(),
            EventType::ActionMe => "ACTION_ME".to_string(),
            EventType::Connect => "CONNECT".to_string(),
            EventType::Ctcp => "CTCP".to_string(),
            EventType::CtcpMe => "CTCP_ME".to_string(),
            EventType::CtcpReply => "CTCP_REP".to_string(),
            EventType::Disconnect => "DISCONNECT".to_string(),
            EventType::Invite => "INVITE".to_string(),
            EventType::Join => "JOIN".to_string(),
            EventType::Kick => "KICK".to_string(),
            EventType::Mode => "MODE".to_string(),
            EventType::Names => "NAMES".to_string(),
            EventType::Nick => "NICK".to_string(),
            EventType::Notice => "NOTICE".to_string(),
            EventType::Numeric => "NUMERIC".to_string(),
            EventType::Part => "PART".to_string(),
            EventType::Pong => "PONG".to_string(),
            EventType::PrivMsg => "PRIVMSG".to_string(),
            EventType::PrivMsgMe => "PRIVMSG_ME".to_string(),
            EventType::Quit => "QUIT".to_string(),
            EventType::Topic => "TOPIC".to_string(),
            EventType::Unknown => "UNKNOWN".to_string(),
            EventType::Whois => "WHOIS".to_string(),
        }
    }
}

impl FromStr for EventType {
    type Err = ParseEventTypeError;

    fn from_str(s: &str) -> Result<Self, ParseEventTypeError> {
        match &s.to_ascii_uppercase()[..] {
            "ACTION" => Ok(EventType::Action),
            "ACTION_ME" => Ok(EventType::ActionMe),
            "CONNECT" => Ok(EventType::Connect),
            "CTCP" => Ok(EventType::Ctcp),
            "CTCP_ME" => Ok(EventType::CtcpMe),
            "CTCP_REP" => Ok(EventType::CtcpReply),
            "DISCONNECT" => Ok(EventType::Disconnect),
            "INVITE" => Ok(EventType::Invite),
            "JOIN" => Ok(EventType::Join),
            "KICK" => Ok(EventType::Kick),
            "MODE" => Ok(EventType::Mode),
            "NAMES" => Ok(EventType::Names),
            "NICK" => Ok(EventType::Nick),
            "NOTICE" => Ok(EventType::Notice),
            "NUMERIC" => Ok(EventType::Numeric),
            "PART" => Ok(EventType::Part),
            "PONG" => Ok(EventType::Pong),
            "PRIVMSG" => Ok(EventType::PrivMsg),
            "PRIVMSG_ME" => Ok(EventType::PrivMsgMe),
            "QUIT" => Ok(EventType::Quit),
            "TOPIC" => Ok(EventType::Topic),
            "UNKNOWN" => Ok(EventType::Unknown),
            "WHOIS" => Ok(EventType::Whois),
            other if other.len() > 8 => {
                match &other[..7] {
                    "COMMAND" => Ok(EventType::Command(other[8..].to_string())),
                    _ => Err(ParseEventTypeError::new())
                }
            },
            _ => Err(ParseEventTypeError::new())
        }
    }
}

/// An event received from the DaZeus server.
///
/// You can retrieve the parameters from the event using one of three different methods:
///
/// 1. Using the params field directly.
/// 2. Using the `param()` method with an index which will return a string slice.
/// 3. Using indexing on the event struct itself, i.e. `event[0]` to receive the first parameter.
///
/// The prefered method is the last one.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// The type of event that was received.
    pub event: EventType,
    /// The parameters attached to the event.
    pub params: Vec<String>,
}

/// Returns whether or not the given Json data could be a valid event object.
pub fn is_event_json(data: &Json) -> bool {
    data.is_object() && data.as_object().unwrap().contains_key("event")
}

impl Event {
    /// Create a new event based on the basic properties of an event.
    ///
    /// Allows creation of events for testing purposes. Also used internally for constructing
    /// events based on parsed Json objects.
    ///
    /// # Example
    /// ```
    /// Event::new(EventType::PrivMsg, vec!(
    ///    "network".to_string(),
    ///    "sender".to_string(),
    ///    "receiver".to_string(),
    ///    "message".to_string()
    /// ))
    /// ```
    pub fn new(event: EventType, params: Vec<String>) -> Event {
        Event { event: event, params: params }
    }

    /// Create a new event based on a Json data object.
    ///
    /// Typically this method will be called by the bindings itself to create an event instance
    /// from some received json blob from the core.
    pub fn from_json(data: &Json) -> Result<Event, InvalidJsonError> {
        if data.is_object() {
            let obj = data.as_object().unwrap();
            if obj.contains_key("event") && obj.contains_key("params") {
                let evt = obj.get("event").unwrap();
                let params = obj.get("params").unwrap();
                if evt.is_string() && params.is_array() {
                    Event::create_event(&evt.as_string().unwrap(), &params.as_array().unwrap())
                } else {
                    Err(InvalidJsonError::new(""))
                }
            } else {
                Err(InvalidJsonError::new(""))
            }
        } else {
            Err(InvalidJsonError::new(""))
        }
    }

    /// Create a new event based on the properties extracted from the Json.
    fn create_event(evt: &str, params: &Vec<Json>) -> Result<Event, InvalidJsonError> {
        if evt == "COMMAND" {
            if params.len() >= 4 && params[3].is_string() {
                let cmd = params[3].as_string().unwrap().to_string();
                Ok(Event::new(EventType::Command(cmd), Event::param_strs(params)))
            } else {
                Err(InvalidJsonError::new(""))
            }
        } else {
            match EventType::from_str(evt) {
                Ok(evt) => Ok(Event::new(evt, Event::param_strs(params))),
                Err(_) => Err(InvalidJsonError::new(""))
            }
        }
    }

    /// Extract string parameters from an array of `Json::String` objects.
    fn param_strs(params: &Vec<Json>) -> Vec<String> {
        let mut strs = Vec::new();
        for param in params {
            if param.is_string() {
                strs.push(param.as_string().unwrap().to_string());
            }
        }
        strs
    }

    /// Retrieve a parameter from the list of parameters contained in the event.
    pub fn param<'a>(&'a self, idx: usize) -> &'a str {
        &self.params[idx][..]
    }

    /// Retrieve the number of parameters for the event.
    pub fn len(&self) -> usize {
        self.params.len()
    }
}

impl<'b> Index<usize> for Event {
    type Output = str;

    fn index<'a>(&'a self, index: usize) -> &'a str {
        self.param(index)
    }
}

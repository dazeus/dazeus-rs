use std::str::FromStr;
use serialize::json::Json;
use super::error::InvalidJsonError;
use std::ops::Index;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseEventTypeError { _priv: () }

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Action,
    ActionMe,
    Command(String),
    Connect,
    Ctcp,
    CtcpMe,
    CtcpReply,
    Disconnect,
    Invite,
    Join,
    Kick,
    Mode,
    Names,
    Nick,
    Notice,
    Numeric,
    Part,
    Pong,
    PrivMsg,
    PrivMsgMe,
    Quit,
    Topic,
    Unknown,
    Whois,
}

impl ToString for EventType {
    fn to_string(&self) -> String {
        match *self {
            EventType::Command(ref s) => format!("COMMAND_{}", s),
            EventType::Action => String::from_str("ACTION"),
            EventType::ActionMe => String::from_str("ACTION_ME"),
            EventType::Connect => String::from_str("CONNECT"),
            EventType::Ctcp => String::from_str("CTCP"),
            EventType::CtcpMe => String::from_str("CTCP_ME"),
            EventType::CtcpReply => String::from_str("CTCP_REP"),
            EventType::Disconnect => String::from_str("DISCONNECT"),
            EventType::Invite => String::from_str("INVITE"),
            EventType::Join => String::from_str("JOIN"),
            EventType::Kick => String::from_str("KICK"),
            EventType::Mode => String::from_str("MODE"),
            EventType::Names => String::from_str("NAMES"),
            EventType::Nick => String::from_str("NICK"),
            EventType::Notice => String::from_str("NOTICE"),
            EventType::Numeric => String::from_str("NUMERIC"),
            EventType::Part => String::from_str("PART"),
            EventType::Pong => String::from_str("PONG"),
            EventType::PrivMsg => String::from_str("PRIVMSG"),
            EventType::PrivMsgMe => String::from_str("PRIVMSG_ME"),
            EventType::Quit => String::from_str("QUIT"),
            EventType::Topic => String::from_str("TOPIC"),
            EventType::Unknown => String::from_str("UNKNOWN"),
            EventType::Whois => String::from_str("WHOIS"),
        }
    }
}

impl FromStr for EventType {
    type Err = ParseEventTypeError;

    fn from_str(s: &str) -> Result<Self, ParseEventTypeError> {
        match &s.to_uppercase()[..] {
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
                    "COMMAND" => Ok(EventType::Command(String::from_str(&other[8..]))),
                    _ => Err(ParseEventTypeError { _priv: () })
                }
            },
            _ => Err(ParseEventTypeError { _priv: () })
        }
    }
}

/// An event received from the DaZeus server
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub event: EventType,
    pub params: Vec<String>,
}

impl Event {
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

    pub fn for_event(event: EventType, params: Vec<String>) -> Event {
        Event { event: event, params: params }
    }

    fn create_event(evt: &str, params: &Vec<Json>) -> Result<Event, InvalidJsonError> {
        if evt == "COMMAND" {
            if params.len() >= 4 && params[3].is_string() {
                let cmd = String::from_str(params[3].as_string().unwrap());
                Ok(Event { event: EventType::Command(cmd), params: Event::param_strs(params) })
            } else {
                Err(InvalidJsonError::new(""))
            }
        } else {
            match EventType::from_str(evt) {
                Ok(evt) => Ok(Event { event: evt, params: Event::param_strs(params) }),
                Err(_) => Err(InvalidJsonError::new(""))
            }
        }
    }

    fn param_strs(params: &Vec<Json>) -> Vec<String> {
        let mut strs = Vec::new();
        for param in params {
            if param.is_string() {
                strs.push(String::from_str(param.as_string().unwrap()));
            }
        }
        strs
    }

    pub fn is_event(data: &Json) -> bool {
        data.is_object() && data.as_object().unwrap().contains_key("event")
    }

    pub fn param<'a>(&'a self, idx: usize) -> &'a str {
        &self.params[idx][..]
    }
}

impl<'b> Index<usize> for Event {
    type Output = str;

    fn index<'a>(&'a self, index: usize) -> &'a str {
        self.param(index)
    }
}

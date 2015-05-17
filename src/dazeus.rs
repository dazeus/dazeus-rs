use std::io::{Read, Write};
use super::event::{Event, EventType};
use super::handler::{Handler, Message};
use super::listener::{ListenerHandle, Listener};
use super::request::{ConfigGroup, Request};
use super::response::Response;
use super::scope::Scope;
use super::error::{ReceiveError, Error};
use std::cell::RefCell;


/// The base DaZeus struct.
///
/// See the [crate documentation](./index.html) for a more detailed instruction on how to get
/// started with these DaZeus bindings.
pub struct DaZeus<'a, T> {
    handler: RefCell<Handler<T>>,
    listeners: Vec<Listener<'a>>,
    current_handle: u64,
}

impl<'a, T> DaZeus<'a, T> where T: Read + Write {
    /// Create a new instance of DaZeus from the given connection.
    pub fn new(conn: T) -> DaZeus<'a, T> {
        DaZeus {
            handler: RefCell::new(Handler::new(conn)),
            listeners: Vec::new(),
            current_handle: 1
        }
    }

    /// Loop wait for messages to receive in a blocking way.
    pub fn listen(&self) -> Result<(), Error> {
        loop {
            try!(self.try_next_event());
        }
    }

    fn next_response(&self) -> Result<Response, Error> {
        loop {
            let msg = { self.handler.borrow_mut().read() };
            match try!(msg) {
                Message::Event(e) => self.handle_event(e),
                Message::Response(r) => return Ok(r),
            }
        }
    }

    fn try_next_event(&self) -> Result<Event, Error> {
        let msg = { self.handler.borrow_mut().read() };
        match try!(msg) {
            Message::Event(e) => {
                self.handle_event(e.clone());
                Ok(e)
            },
            Message::Response(_) => Err(Error::ReceiveError(ReceiveError::new())),
        }
    }

    fn next_event(&self) -> Event {
        match self.try_next_event() {
            Ok(evt) => evt,
            Err(e) => panic!("{}", e),
        }
    }

    /// Handle an event received by calling all event listeners listening for that event type.
    fn handle_event(&self, event: Event) {
        for listener in self.listeners.iter() {
            if listener.event == event.event {
                listener.call(event.clone(), self);
            }
        }
    }

    /// Subscribe to an event type and call the callback function every time such an event occurs.
    pub fn subscribe<F>(&mut self, event: EventType, callback: F) -> (ListenerHandle, Response)
        where F: FnMut(Event, &DaZeusClient) + 'a
    {
        let request = match event {
            EventType::Command(ref cmd) => Request::SubscribeCommand(cmd.clone(), None),
            _ => Request::Subscribe(event.clone()),
        };

        let handle = self.current_handle;
        self.current_handle += 1;
        let listener = Listener::new(handle, event, callback);

        self.listeners.push(listener);
        (handle, self.send(request))
    }

    /// Subscribe to a command and call the callback function every time such a command occurs.
    pub fn subscribe_command<F>(&mut self, command: &str, callback: F) -> (ListenerHandle, Response)
        where F: FnMut(Event, &DaZeusClient) + 'a
    {
        self.subscribe(EventType::Command(command.to_string()), callback)
    }
}

/// Methods for interaction with the DaZeus server.
pub trait DaZeusClient<'a> {
    /// Try to send a request to DaZeus
    fn try_send(&self, request: Request) -> Result<Response, Error>;

    /// Send a request to DaZeus and retrieve a Future in which the response will be contained.
    fn send(&self, request: Request) -> Response;

    /// Unsubscribe a listener for some event.
    fn unsubscribe(&mut self, handle: ListenerHandle) -> Response;

    /// Remove all subscriptions for a specific event type.
    fn unsubscribe_all(&mut self, event: EventType) -> Response;

    /// Check if there is any active listener for the given event type.
    fn has_any_subscription(&self, event: EventType) -> bool;

    /// Retrieve the networks the bot is connected to.
    fn networks(&self) -> Response;

    /// Retrieve the channels the bot is in for a given network.
    fn channels(&self, network: &str) -> Response;

    /// Send a message to a specific channel using the PRIVMSG method.
    fn message(&self, network: &str, channel: &str, message: &str) -> Response;

    /// Send a CTCP NOTICE to a specific channel.
    fn notice(&self, network: &str, channel: &str, message: &str) -> Response;

    /// Send a CTCP REQUEST to a specific channel.
    fn ctcp(&self, network: &str, channel: &str, message: &str) -> Response;

    /// Send a CTCP REPLY to a specific channel.
    fn ctcp_reply(&self, network: &str, channel: &str, message: &str) -> Response;

    /// Send a CTCP ACTION to a specific channel
    fn action(&self, network: &str, channel: &str, message: &str) -> Response;

    /// Send a request for the list of nicks in a channel.
    ///
    /// Note that the response will not contain the names data, instead listen for a names event.
    /// The Response will only indicate whether or not the request has been submitted successfully.
    /// The server may respond with an `EventType::Names` event any time after this request has
    /// been submitted.
    fn send_names(&self, network: &str, channel: &str) -> Response;

    /// Send a request for a whois of a specific nick on some network.
    ///
    /// Note that the response will not contain the whois data, instead listen for a whois event.
    /// The Response will only indicate whether or not the request has been submitted successfully.
    /// The server may respond with an `EventType::Whois` event any time after this request has
    /// been submitted.
    fn send_whois(&self, network: &str, nick: &str) -> Response;

    /// Try to join a channel on some network.
    fn join(&self, network: &str, channel: &str) -> Response;

    /// Try to leave a channel on some network.
    fn part(&self, network: &str, channel: &str) -> Response;

    /// Retrieve the nickname of the bot on the given network.
    fn nick(&self, network: &str) -> Response;

    /// Send a handshake to the DaZeus core.
    fn handshake(&self, name: &str, version: &str, config: Option<&str>) -> Response;

    /// Retrieve a config value from the DaZeus config.
    fn get_config(&self, name: &str, group: ConfigGroup) -> Response;

    /// Retrieve the character that is used by the bot for highlighting.
    fn get_highlight_char(&self) -> Response;

    /// Retrieve a property stored in the bot database.
    fn get_property(&self, name: &str, scope: Scope) -> Response;

    /// Set a property to be stored in the bot database.
    fn set_property(&self, name: &str, value: &str, scope: Scope) -> Response;

    /// Remove a property stored in the bot database.
    fn unset_property(&self, name: &str, scope: Scope) -> Response;

    /// Retrieve a list of keys starting with the common prefix with the given scope.
    fn get_property_keys(&self, prefix: &str, scope: Scope) -> Response;

    /// Set a permission to either allow or deny for a specific scope.
    fn set_permission(&self, permission: &str, allow: bool, scope: Scope) -> Response;

    /// Retrieve whether for some scope the given permission was set.
    ///
    /// Will return the default if it was not.
    fn has_permission(&self, permission: &str, default: bool, scope: Scope) -> Response;

    /// Remove a set permission from the bot.
    fn unset_permission(&self, permission: &str, scope: Scope) -> Response;

    /// Send a whois request and wait for an event that answers this request (blocking).
    ///
    /// Note that the IRC server may not respond to the whois request (if it has been configured
    /// this way), in which case this request will block forever.
    fn whois(&mut self, network: &str, nick: &str) -> Event;

    /// Send a names request and wait for an event that answers this request (blocking).
    ///
    /// Note that the IRC server may not respond to the names request (if it has been configured
    /// this way), in which case this request will block forever.
    fn names(&mut self, network: &str, channel: &str) -> Event;

    /// Send a reply in response to some event.
    ///
    /// Note that not all types of events can be responded to. Mostly message type events
    /// concerning some IRC user can be responded to. Join events can also be responded to.
    fn reply(&self, event: &Event, message: &str, highlight: bool) -> Response;

    /// Send a reply (as a notice) in response to some event.
    ///
    /// Note that not all types of events can be responded to. Mostly message type events
    /// concerning some IRC user can be responded to. Join events can also be responded to.
    fn reply_with_notice(&self, event: &Event, message: &str) -> Response;

    /// Send a reply (as a ctcp action) in response to some event.
    ///
    /// Note that not all types of events can be responded to. Mostly message type events
    /// concerning some IRC user can be responded to. Join events can also be responded to.
    fn reply_with_action(&self, event: &Event, message: &str) -> Response;
}

impl<'a, T> DaZeusClient<'a> for DaZeus<'a, T> where T: Read + Write {
    /// Try to send a request to DaZeus
    fn try_send(&self, request: Request) -> Result<Response, Error> {
        { try!(self.handler.borrow_mut().write(request)) };
        self.next_response()
    }

    /// Send a request to DaZeus and retrieve a Future in which the response will be contained.
    fn send(&self, request: Request) -> Response {
        match self.try_send(request) {
            Ok(response) => response,
            Err(e) => panic!("{}", e),
        }
    }

    /// Unsubscribe a listener for some event.
    fn unsubscribe(&mut self, handle: ListenerHandle) -> Response {
        // first find the event type
        let event = {
            match self.listeners.iter().find(|&ref l| l.has_handle(handle)) {
                Some(listener) => Some(listener.event.clone()),
                None => None,
            }
        };

        self.listeners.retain(|&ref l| !l.has_handle(handle));
        match event {
            // we can't unsubscribe commands
            Some(EventType::Command(_)) => Response::for_success(),

            // unsubscribe if there are no more listeners for the event
            Some(evt) => match self.listeners.iter().any(|&ref l| l.event == evt) {
                false => self.send(Request::Unsubscribe(evt)),
                true => Response::for_success(),
            },

            None => Response::for_fail("Could not find listener with given handle"),
        }
    }

    /// Remove all subscriptions for a specific event type.
    fn unsubscribe_all(&mut self, event: EventType) -> Response {
        self.listeners.retain(|&ref l| l.event != event);
        match event {
            EventType::Command(_) => Response::for_success(),
            _ => self.send(Request::Unsubscribe(event)),
        }
    }

    /// Check if there is any active listener for the given event type.
    fn has_any_subscription(&self, event: EventType) -> bool {
        self.listeners.iter().any(|&ref l| l.event == event)
    }

    /// Retrieve the networks the bot is connected to.
    fn networks(&self) -> Response {
        self.send(Request::Networks)
    }

    /// Retrieve the channels the bot is in for a given network.
    fn channels(&self, network: &str) -> Response {
        self.send(Request::Channels(network.to_string()))
    }

    /// Send a message to a specific channel using the PRIVMSG method.
    fn message(&self, network: &str, channel: &str, message: &str) -> Response {
        self.send(Request::Message(
            network.to_string(),
            channel.to_string(),
            message.to_string()
        ))
    }

    /// Send a CTCP NOTICE to a specific channel.
    fn notice(&self, network: &str, channel: &str, message: &str) -> Response {
        self.send(Request::Notice(
            network.to_string(),
            channel.to_string(),
            message.to_string()
        ))
    }

    /// Send a CTCP REQUEST to a specific channel.
    fn ctcp(&self, network: &str, channel: &str, message: &str) -> Response {
        self.send(Request::Ctcp(
            network.to_string(),
            channel.to_string(),
            message.to_string()
        ))
    }

    /// Send a CTCP REPLY to a specific channel.
    fn ctcp_reply(&self, network: &str, channel: &str, message: &str) -> Response {
        self.send(Request::CtcpReply(
            network.to_string(),
            channel.to_string(),
            message.to_string()
        ))
    }

    /// Send a CTCP ACTION to a specific channel
    fn action(&self, network: &str, channel: &str, message: &str) -> Response {
        self.send(Request::Action(
            network.to_string(),
            channel.to_string(),
            message.to_string()
        ))
    }

    /// Send a request for the list of nicks in a channel.
    ///
    /// Note that the response will not contain the names data, instead listen for a names event.
    /// The Response will only indicate whether or not the request has been submitted successfully.
    /// The server may respond with an `EventType::Names` event any time after this request has
    /// been submitted.
    fn send_names(&self, network: &str, channel: &str) -> Response {
        self.send(Request::Names(network.to_string(), channel.to_string()))
    }

    /// Send a request for a whois of a specific nick on some network.
    ///
    /// Note that the response will not contain the whois data, instead listen for a whois event.
    /// The Response will only indicate whether or not the request has been submitted successfully.
    /// The server may respond with an `EventType::Whois` event any time after this request has
    /// been submitted.
    fn send_whois(&self, network: &str, nick: &str) -> Response {
        self.send(Request::Whois(network.to_string(), nick.to_string()))
    }

    /// Try to join a channel on some network.
    fn join(&self, network: &str, channel: &str) -> Response {
        self.send(Request::Join(network.to_string(), channel.to_string()))
    }

    /// Try to leave a channel on some network.
    fn part(&self, network: &str, channel: &str) -> Response {
        self.send(Request::Part(network.to_string(), channel.to_string()))
    }

    /// Retrieve the nickname of the bot on the given network.
    fn nick(&self, network: &str) -> Response {
        self.send(Request::Nick(network.to_string()))
    }

    /// Send a handshake to the DaZeus core.
    fn handshake(&self, name: &str, version: &str, config: Option<&str>) -> Response {
        let n = name.to_string();
        let v = version.to_string();
        let req = match config {
            Some(config_name) => Request::Handshake(n, v, Some(config_name.to_string())),
            None => Request::Handshake(n, v, None),
        };
        self.send(req)
    }

    /// Retrieve a config value from the DaZeus config.
    fn get_config(&self, name: &str, group: ConfigGroup) -> Response {
        self.send(Request::Config(name.to_string(), group))
    }

    /// Retrieve the character that is used by the bot for highlighting.
    fn get_highlight_char(&self) -> Response {
        self.get_config("highlight", ConfigGroup::Core)
    }

    /// Retrieve a property stored in the bot database.
    fn get_property(&self, name: &str, scope: Scope) -> Response {
        self.send(Request::GetProperty(name.to_string(), scope))
    }

    /// Set a property to be stored in the bot database.
    fn set_property(&self, name: &str, value: &str, scope: Scope) -> Response {
        self.send(Request::SetProperty(name.to_string(), value.to_string(), scope))
    }

    /// Remove a property stored in the bot database.
    fn unset_property(&self, name: &str, scope: Scope) -> Response {
        self.send(Request::UnsetProperty(name.to_string(), scope))
    }

    /// Retrieve a list of keys starting with the common prefix with the given scope.
    fn get_property_keys(&self, prefix: &str, scope: Scope) -> Response {
        self.send(Request::PropertyKeys(prefix.to_string(), scope))
    }

    /// Set a permission to either allow or deny for a specific scope.
    fn set_permission(&self, permission: &str, allow: bool, scope: Scope) -> Response {
        self.send(Request::SetPermission(permission.to_string(), allow, scope))
    }

    /// Retrieve whether for some scope the given permission was set.
    ///
    /// Will return the default if it was not.
    fn has_permission(&self, permission: &str, default: bool, scope: Scope) -> Response {
        self.send(Request::HasPermission(permission.to_string(), default, scope))
    }

    /// Remove a set permission from the bot.
    fn unset_permission(&self, permission: &str, scope: Scope) -> Response {
        self.send(Request::UnsetPermission(permission.to_string(), scope))
    }

    /// Send a whois request and wait for an event that answers this request (blocking).
    ///
    /// Note that the IRC server may not respond to the whois request (if it has been configured
    /// this way), in which case this request will block forever.
    fn whois(&mut self, network: &str, nick: &str) -> Event {
        if !self.has_any_subscription(EventType::Whois) {
            self.send(Request::Subscribe(EventType::Whois));
        }
        self.send_whois(network, nick);

        loop {
            let evt = self.next_event();
            match evt.event {
                EventType::Whois if &evt[0] == network && &evt[2] == nick => {
                    if !self.has_any_subscription(EventType::Whois) {
                        self.send(Request::Unsubscribe(EventType::Whois));
                    }
                    return evt;
                },
                _ => (),
            }
        }
    }

    /// Send a names request and wait for an event that answers this request (blocking).
    ///
    /// Note that the IRC server may not respond to the names request (if it has been configured
    /// this way), in which case this request will block forever.
    fn names(&mut self, network: &str, channel: &str) -> Event {
        if !self.has_any_subscription(EventType::Names) {
            self.send(Request::Subscribe(EventType::Names));
        }
        self.send_names(network, channel);

        loop {
            let evt = self.next_event();
            match evt.event {
                EventType::Names if &evt[0] == network && &evt[2] == channel => {
                    if !self.has_any_subscription(EventType::Names) {
                        self.send(Request::Unsubscribe(EventType::Names));
                    }
                    return evt;
                },
                _ => (),
            }
        }
    }

    /// Send a reply in response to some event.
    ///
    /// Note that not all types of events can be responded to. Mostly message type events
    /// concerning some IRC user can be responded to. Join events can also be responded to.
    fn reply(&self, event: &Event, message: &str, highlight: bool) -> Response {
        if let Some((network, channel, user)) = targets_for_event(event) {
            let resp = self.nick(network);
            let nick = resp.get_str_or("nick", "");
            if channel == nick {
                self.message(network, user, message)
            } else {
                if highlight {
                    let msg = format!("{}: {}", user, message);
                    self.message(network, channel, &msg[..])
                } else {
                    self.message(network, channel, message)
                }
            }
        } else {
            Response::for_fail("Not an event to reply to")
        }
    }

    /// Send a reply (as a notice) in response to some event.
    ///
    /// Note that not all types of events can be responded to. Mostly message type events
    /// concerning some IRC user can be responded to. Join events can also be responded to.
    fn reply_with_notice(&self, event: &Event, message: &str) -> Response {
        if let Some((network, channel, user)) = targets_for_event(event) {
            let resp = self.nick(network);
            let nick = resp.get_str_or("nick", "");
            if channel == nick {
                self.notice(network, user, message)
            } else {
                self.notice(network, channel, message)
            }
        } else {
            Response::for_fail("Not an event to reply to")
        }
    }

    /// Send a reply (as a ctcp action) in response to some event.
    ///
    /// Note that not all types of events can be responded to. Mostly message type events
    /// concerning some IRC user can be responded to. Join events can also be responded to.
    fn reply_with_action(&self, event: &Event, message: &str) -> Response {
        if let Some((network, channel, user)) = targets_for_event(event) {
            let resp = self.nick(network);
            let nick = resp.get_str_or("nick", "");
            if channel == nick {
                self.action(network, user, message)
            } else {
                self.action(network, channel, message)
            }
        } else {
            Response::for_fail("Not an event to reply to")
        }
    }
}

fn targets_for_event(event: &Event) -> Option<(&str, &str, &str)> {
    let params = &event.params;
    match event.event {
        EventType::Join
        | EventType::PrivMsg
        | EventType::Notice
        | EventType::Ctcp
        | EventType::Command(_)
        | EventType::Action => Some((&params[0][..], &params[2][..], &params[1][..])),
        _ => None,
    }
}

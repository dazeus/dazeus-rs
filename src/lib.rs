//! DaZeus IRC bot bindings for rust
#![feature(io, std_misc, collections, slice_patterns)]

#[macro_use]
extern crate log;
extern crate rustc_serialize as serialize;
extern crate unix_socket;

pub use self::connection::connection_from_str;
pub use self::event::*;
pub use self::listener::ListenerHandle;
pub use self::request::*;
pub use self::response::*;
pub use self::scope::*;
use self::connection::Connection;
use self::handlers::{Reader, Writer, Handler};
use self::listener::Listener;
use std::cell::{Cell, RefCell};
use std::io::{Read, Write, BufStream};
use std::sync::Future;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;


mod connection;
mod event;
mod handlers;
mod listener;
mod request;
mod response;
mod scope;
pub mod error;


/// Methods that need to be implemented for sending commands to the server
pub trait Commander {
    fn subscribe<F>(&self, event: EventType, callback: F) -> (ListenerHandle, Future<Response>) where F: FnMut(Event);
    fn subscribe_command<F>(&self, command: &str, callback: F) -> (ListenerHandle, Future<Response>) where F: FnMut(Event);
    fn unsubscribe(&self, handle: ListenerHandle) -> Future<Response>;
    fn unsubscribe_all(&self, event: EventType) -> Future<Response>;
    fn has_any_subscription(&self, event: EventType) -> bool;
    fn networks(&self) -> Future<Response>;
    fn channels(&self, network: &str) -> Future<Response>;
    fn message(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn notice(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn ctcp(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn ctcp_reply(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn action(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn send_names(&self, network: &str, channel: &str) -> Future<Response>;
    fn send_whois(&self, network: &str, nick: &str) -> Future<Response>;
    fn join(&self, network: &str, channel: &str) -> Future<Response>;
    fn part(&self, network: &str, channel: &str) -> Future<Response>;
    fn nick(&self, network: &str) -> Future<Response>;
    fn handshake(&self, name: &str, version: &str, config: Option<&str>) -> Future<Response>;
    fn get_config(&self, group: &str, name: Option<&str>) -> Future<Response>;
    fn get_highlight_char(&self) -> Future<Response>;
    fn get_property(&self, name: &str, scope: Scope) -> Future<Response>;
    fn set_property(&self, name: &str, value: &str, scope: Scope) -> Future<Response>;
    fn unset_property(&self, name: &str, scope: Scope) -> Future<Response>;
    fn get_property_keys(&self, prefix: &str, scope: Scope) -> Future<Response>;
    fn set_permission(&self, permission: &str, allow: bool, scope: Scope) -> Future<Response>;
    fn has_permission(&self, permission: &str, default: bool, scope: Scope) -> Future<Response>;
    fn unset_permission(&self, permission: &str, scope: Scope) -> Future<Response>;
    fn whois(&self, network: &str, nick: &str) -> Event;
    fn names(&self, network: &str, channel: &str) -> Event;
    fn reply(&self, event: &Event, message: &str, highlight: bool) -> Future<Response>;
    fn reply_with_action(&self, event: &Event, message: &str) -> Future<Response>;
    fn reply_with_notice(&self, event: &Event, message: &str) -> Future<Response>;
    fn listen(&self);
}

/// The base DaZeus struct
pub struct DaZeus<'a> {
    event_rx: Receiver<Event>,
    request_tx: Sender<(Request, Sender<Response>)>,
    listeners: RefCell<Vec<Listener<'a>>>,
    current_handle: Cell<ListenerHandle>,
}


impl<'a> DaZeus<'a> {
    /// Create a new instance of DaZeus from the given connection
    pub fn from_conn(conn: Connection) -> DaZeus<'a> {
        let clone = conn.try_clone().unwrap();
        DaZeus::new(conn, clone)
    }

    /// Create a new instance of DaZeus from the given connection, making use of a buffered stream
    pub fn from_conn_buff(conn: Connection) -> DaZeus<'a> {
        let clone = conn.try_clone().unwrap();
        DaZeus::new(BufStream::new(conn), BufStream::new(clone))
    }

    /// Create a new instance from a Read and Send, note that both need to point to the same socket
    pub fn new<R: Read + Send + 'static, W: Write + Send + 'static>(read: R, write: W) -> DaZeus<'a> {
        let (read_tx, read_rx) = channel();
        let (write_tx, write_rx) = channel();
        let mut reader = Reader::new(read, read_tx);
        let mut writer = Writer::new(write, write_rx);

        let (event_tx, event_rx) = channel();
        let (request_tx, request_rx) = channel();
        let mut handler = Handler::new(write_tx, event_tx);

        thread::spawn(move || { reader.run(); });
        thread::spawn(move || { writer.run(); });
        thread::spawn(move || { handler.run(read_rx, request_rx); });

        DaZeus {
            event_rx: event_rx,
            request_tx: request_tx,
            listeners: RefCell::new(Vec::new()),
            current_handle: Cell::new(1),
        }
    }

    /// Send a new Json packet to DaZeus and return the channel on which a response will be written
    fn send_to(&self, data: Request) -> Receiver<Response> {
        let (response_tx, response_rx) = channel();
        match self.request_tx.send((data, response_tx)) {
            Err(e) => panic!(e),
            Ok(_) => (),
        }
        response_rx
    }

    /// Send a new Json packet to DaZeus and retrieve a Future for responding
    pub fn send(&self, data: Request) -> Future<Response> {
        Future::from_receiver(self.send_to(data))
    }

    /// Handle an event received
    fn handle_event(&self, event: Event) {
        for listener in self.listeners.borrow_mut().iter_mut() {
            if listener.event == event.event {
                listener.call(event.clone());
            }
        }
    }

    /// Retrieve the next event (blocking)
    /// All listeners will have been called already
    pub fn next_event(&self) -> Event {
        match self.event_rx.recv() {
            Ok(event) => {
                self.handle_event(event.clone());
                event
            },
            Err(e) => panic!(e),
        }
    }
}


impl<'a> Commander for DaZeus<'a> {
    /// Loop wait for messages to receive in a blocking way
    fn listen(&self) {
        loop {
            let _ = self.next_event();
        }
    }

    /// Subscribe to an event type and call the callback function every time such an event occurs
    fn subscribe<F>(&self, event: EventType, callback: F) -> (ListenerHandle, Future<Response>)
        where F: FnMut(Event) + 'a
    {
        let request = match event {
            EventType::Command(ref cmd) => Request::SubscribeCommand(cmd.clone(), None),
            _ => Request::Subscribe(event.to_string()),
        };

        let handle = self.current_handle.get();
        self.current_handle.set(handle + 1);
        let listener = Listener::new(handle, event, callback);

        let mut listeners = self.listeners.borrow_mut();
        listeners.push(listener);
        (handle, self.send(request))
    }

    /// Subscribe to a command and call the callback function every time such a command occurs
    fn subscribe_command<F>(&self, command: &str, callback: F) -> (ListenerHandle, Future<Response>)
        where F: FnMut(Event) + 'a
    {
        self.subscribe(EventType::Command(String::from_str(command)), callback)
    }

    /// Unsubscribe a listener for some event
    fn unsubscribe(&self, handle: ListenerHandle) -> Future<Response> {
        let mut listeners = self.listeners.borrow_mut();

        // first find the event type
        let event = {
            match listeners.iter().find(|&ref l| l.has_handle(handle)) {
                Some(listener) => Some(listener.event.clone()),
                None => None,
            }
        };

        listeners.retain(|&ref l| !l.has_handle(handle));
        match event {
            // we can't unsubscribe commands
            Some(EventType::Command(_)) => Future::from_value(Response::for_success()),

            // unsubscribe if there are no more listeners for the event
            Some(evt) => match listeners.iter().any(|&ref l| l.event == evt) {
                false => self.send(Request::Unsubscribe(evt.to_string())),
                true => Future::from_value(Response::for_success()),
            },

            None => Future::from_value(Response::from_fail("Could not find listener with given handle")),
        }
    }

    /// Remove all subscriptions for a specific event type
    fn unsubscribe_all(&self, event: EventType) -> Future<Response> {
        let mut listeners = self.listeners.borrow_mut();
        listeners.retain(|&ref l| l.event != event);
        match event {
            EventType::Command(_) => Future::from_value(Response::for_success()),
            _ => self.send(Request::Unsubscribe(event.to_string())),
        }
    }

    /// Check if there is any active listener for the given event type
    fn has_any_subscription(&self, event: EventType) -> bool {
        self.listeners.borrow().iter().any(|&ref l| l.event == event)
    }

    /// Retrieve the networks the bot is connected to
    fn networks(&self) -> Future<Response> {
        self.send(Request::Networks)
    }

    /// Retrieve the channels the bot is in for a given network
    fn channels(&self, network: &str) -> Future<Response> {
        self.send(Request::Channels(String::from_str(network)))
    }

    /// Send a message to a specific channel using the PRIVMSG method
    fn message(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.send(Request::Message(
            String::from_str(network),
            String::from_str(channel),
            String::from_str(message)
        ))
    }

    /// Send a CTCP NOTICE to a specific channel
    fn notice(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.send(Request::Notice(
            String::from_str(network),
            String::from_str(channel),
            String::from_str(message)
        ))
    }

    /// Send a CTCP REQUEST to a specific channel
    fn ctcp(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.send(Request::Ctcp(
            String::from_str(network),
            String::from_str(channel),
            String::from_str(message)
        ))
    }

    /// Send a CTCP REPLY to a specific channel
    fn ctcp_reply(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.send(Request::CtcpReply(
            String::from_str(network),
            String::from_str(channel),
            String::from_str(message)
        ))
    }

    /// Send a CTCP ACTION to a specific channel
    fn action(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.send(Request::Action(
            String::from_str(network),
            String::from_str(channel),
            String::from_str(message)
        ))
    }

    /// Send a request for the list of nicks in a channel
    /// Note that the response will not contain the names data, instead listen for a names event
    fn send_names(&self, network: &str, channel: &str) -> Future<Response> {
        self.send(Request::Names(String::from_str(network), String::from_str(channel)))
    }

    /// Send a request for a whois of a specific nick on some network
    /// Note that the response will not contain the whois data, instead listen for a whois event
    fn send_whois(&self, network: &str, nick: &str) -> Future<Response> {
        self.send(Request::Whois(String::from_str(network), String::from_str(nick)))
    }

    /// Try to join a channel on some network
    fn join(&self, network: &str, channel: &str) -> Future<Response> {
        self.send(Request::Join(String::from_str(network), String::from_str(channel)))
    }

    /// Try to leave a channel on some network
    fn part(&self, network: &str, channel: &str) -> Future<Response> {
        self.send(Request::Part(String::from_str(network), String::from_str(channel)))
    }

    /// Retrieve the nickname of the bot on the given network
    fn nick(&self, network: &str) -> Future<Response> {
        self.send(Request::Nick(String::from_str(network)))
    }

    /// Send a handshake to the DaZeus server
    fn handshake(&self, name: &str, version: &str, config: Option<&str>) -> Future<Response> {
        let n = String::from_str(name);
        let v = String::from_str(version);
        let req = match config {
            Some(config_name) => Request::Handshake(n, v, Some(String::from_str(config_name))),
            None => Request::Handshake(n, v, None),
        };
        self.send(req)
    }

    /// Retrieve a config value from the DaZeus server config
    fn get_config(&self, name: &str, group: Option<&str>) -> Future<Response> {
        let n = String::from_str(name);
        let req = match group {
            Some(group_name) => Request::Config(n, Some(String::from_str(group_name))),
            None => Request::Config(n, None),
        };
        self.send(req)
    }

    /// Retrieve the character that is used by the bot for highlighting
    fn get_highlight_char(&self) -> Future<Response> {
        self.get_config("highlight", Some("core"))
    }

    /// Retrieve a property stored in the bot database
    fn get_property(&self, name: &str, scope: Scope) -> Future<Response> {
        self.send(Request::GetProperty(String::from_str(name), scope))
    }

    /// Set a property to be stored in the bot database
    fn set_property(&self, name: &str, value: &str, scope: Scope) -> Future<Response> {
        self.send(Request::SetProperty(String::from_str(name), String::from_str(value), scope))
    }

    /// Remove a property stored in the bot database
    fn unset_property(&self, name: &str, scope: Scope) -> Future<Response> {
        self.send(Request::UnsetProperty(String::from_str(name), scope))
    }

    /// Retrieve a list of keys starting with the common prefix with the given scope
    fn get_property_keys(&self, prefix: &str, scope: Scope) -> Future<Response> {
        self.send(Request::PropertyKeys(String::from_str(prefix), scope))
    }

    /// Set a permission to either allow or deny for a specific scope
    fn set_permission(&self, permission: &str, allow: bool, scope: Scope) -> Future<Response> {
        self.send(Request::SetPermission(String::from_str(permission), allow, scope))
    }

    /// Retrieve whether for some scope the given permission was set
    /// Will return the default if it was not
    fn has_permission(&self, permission: &str, default: bool, scope: Scope) -> Future<Response> {
        self.send(Request::HasPermission(String::from_str(permission), default, scope))
    }

    /// Remove a set permission from the bot
    fn unset_permission(&self, permission: &str, scope: Scope) -> Future<Response> {
        self.send(Request::UnsetPermission(String::from_str(permission), scope))
    }

    /// Send a whois request and wait for an event that answers this request (blocking)
    fn whois(&self, network: &str, nick: &str) -> Event {
        if !self.has_any_subscription(EventType::Whois) {
            self.send(Request::Subscribe(EventType::Whois.to_string()));
        }
        self.send_whois(network, nick);

        loop {
            let evt = self.next_event();
            match evt.event {
                EventType::Whois if &evt[0] == network && &evt[2] == nick => {
                    if !self.has_any_subscription(EventType::Whois) {
                        self.send(Request::Unsubscribe(EventType::Whois.to_string()));
                    }
                    return evt;
                },
                _ => (),
            }
        }
    }

    /// Send a names request and wait for an event that answers this request (blocking)
    fn names(&self, network: &str, channel: &str) -> Event {
        if !self.has_any_subscription(EventType::Names) {
            self.send(Request::Subscribe(EventType::Names.to_string()));
        }
        self.send_names(network, channel);

        loop {
            let evt = self.next_event();
            match evt.event {
                EventType::Names if &evt[0] == network && &evt[2] == channel => {
                    if !self.has_any_subscription(EventType::Names) {
                        self.send(Request::Unsubscribe(EventType::Names.to_string()));
                    }
                    return evt;
                },
                _ => (),
            }
        }
    }

    /// Send a reply in response to some event
    fn reply(&self, event: &Event, message: &str, highlight: bool) -> Future<Response> {
        if let Some((network, channel, user)) = targets_for_event(event) {
            let resp = self.nick(network).into_inner();
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
            Future::from_value(Response::from_fail("Not an event to reply to"))
        }
    }

    /// Send a reply (as a notice) in response to some event
    fn reply_with_notice(&self, event: &Event, message: &str) -> Future<Response> {
        if let Some((network, channel, user)) = targets_for_event(event) {
            let resp = self.nick(network).into_inner();
            let nick = resp.get_str_or("nick", "");
            if channel == nick {
                self.notice(network, user, message)
            } else {
                self.notice(network, channel, message)
            }
        } else {
            Future::from_value(Response::from_fail("Not an event to reply to"))
        }
    }

    /// Send a reply (as a ctcp action) in response to some event
    fn reply_with_action(&self, event: &Event, message: &str) -> Future<Response> {
        if let Some((network, channel, user)) = targets_for_event(event) {
            let resp = self.nick(network).into_inner();
            let nick = resp.get_str_or("nick", "");
            if channel == nick {
                self.action(network, user, message)
            } else {
                self.action(network, channel, message)
            }
        } else {
            Future::from_value(Response::from_fail("Not an event to reply to"))
        }
    }
}


impl<'a> Commander for RefCell<DaZeus<'a>> {
    /// Loop wait for messages to receive in a blocking way
    fn listen(&self) {
        self.borrow().listen()
    }

    /// Subscribe to an event type and call the callback function every time such an event occurs
    fn subscribe<F>(&self, event: EventType, callback: F) -> (ListenerHandle, Future<Response>)
        where F: FnMut(Event) + 'a
    {
        self.borrow().subscribe(event, callback)
    }

    /// Subscribe to a command and call the callback function every time such a command occurs
    fn subscribe_command<F>(&self, command: &str, callback: F) -> (ListenerHandle, Future<Response>)
        where F: FnMut(Event) + 'a
    {
        self.borrow().subscribe_command(command, callback)
    }

    /// Unsubscribe a listener for some event
    fn unsubscribe(&self, handle: ListenerHandle) -> Future<Response> {
        self.borrow().unsubscribe(handle)
    }

    /// Remove all subscriptions for a specific event type
    fn unsubscribe_all(&self, event: EventType) -> Future<Response> {
        self.borrow().unsubscribe_all(event)
    }

    /// Check if there is any active listener for the given event type
    fn has_any_subscription(&self, event: EventType) -> bool {
        self.borrow().has_any_subscription(event)
    }

    /// Retrieve the networks the bot is connected to
    fn networks(&self) -> Future<Response> {
        self.borrow().networks()
    }

    /// Retrieve the channels the bot is in for a given network
    fn channels(&self, network: &str) -> Future<Response> {
        self.borrow().channels(network)
    }

    /// Send a message to a specific channel using the PRIVMSG method
    fn message(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.borrow().message(network, channel, message)
    }

    /// Send a CTCP NOTICE to a specific channel
    fn notice(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.borrow().notice(network, channel, message)
    }

    /// Send a CTCP REQUEST to a specific channel
    fn ctcp(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.borrow().ctcp(network, channel, message)
    }

    /// Send a CTCP REPLY to a specific channel
    fn ctcp_reply(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.borrow().ctcp_reply(network, channel, message)
    }

    /// Send a CTCP ACTION to a specific channel
    fn action(&self, network: &str, channel: &str, message: &str) -> Future<Response> {
        self.borrow().action(network, channel, message)
    }

    /// Send a request for the list of nicks in a channel
    /// Note that the response will not contain the names data, instead listen for a names event
    fn send_names(&self, network: &str, channel: &str) -> Future<Response> {
        self.borrow().send_names(network, channel)
    }

    /// Send a request for a whois of a specific nick on some network
    /// Note that the response will not contain the whois data, instead listen for a whois event
    fn send_whois(&self, network: &str, nick: &str) -> Future<Response> {
        self.borrow().send_whois(network, nick)
    }

    /// Try to join a channel on some network
    fn join(&self, network: &str, channel: &str) -> Future<Response> {
        self.borrow().join(network, channel)
    }

    /// Try to leave a channel on some network
    fn part(&self, network: &str, channel: &str) -> Future<Response> {
        self.borrow().part(network, channel)
    }

    /// Retrieve the nickname of the bot on the given network
    fn nick(&self, network: &str) -> Future<Response> {
        self.borrow().nick(network)
    }

    /// Send a handshake to the DaZeus server
    fn handshake(&self, name: &str, version: &str, config: Option<&str>) -> Future<Response> {
        self.borrow().handshake(name, version, config)
    }

    /// Retrieve a config value from the DaZeus server config
    fn get_config(&self, name: &str, group: Option<&str>) -> Future<Response> {
        self.borrow().get_config(name, group)
    }

    /// Retrieve the character that is used by the bot for highlighting
    fn get_highlight_char(&self) -> Future<Response> {
        self.borrow().get_highlight_char()
    }

    /// Retrieve a property stored in the bot database
    fn get_property(&self, name: &str, scope: Scope) -> Future<Response> {
        self.borrow().get_property(name, scope)
    }

    /// Set a property to be stored in the bot database
    fn set_property(&self, name: &str, value: &str, scope: Scope) -> Future<Response> {
        self.borrow().set_property(name, value, scope)
    }

    /// Remove a property stored in the bot database
    fn unset_property(&self, name: &str, scope: Scope) -> Future<Response> {
        self.borrow().unset_property(name, scope)
    }

    /// Retrieve a list of keys starting with the common prefix with the given scope
    fn get_property_keys(&self, prefix: &str, scope: Scope) -> Future<Response> {
        self.borrow().get_property_keys(prefix, scope)
    }

    /// Set a permission to either allow or deny for a specific scope
    fn set_permission(&self, permission: &str, allow: bool, scope: Scope) -> Future<Response> {
        self.borrow().set_permission(permission, allow, scope)
    }

    /// Retrieve whether for some scope the given permission was set
    /// Will return the default if it was not
    fn has_permission(&self, permission: &str, default: bool, scope: Scope) -> Future<Response> {
        self.borrow().has_permission(permission, default, scope)
    }

    /// Remove a set permission from the bot
    fn unset_permission(&self, permission: &str, scope: Scope) -> Future<Response> {
        self.borrow().unset_permission(permission, scope)
    }

    /// Send a whois request and wait for an event that answers this request (blocking)
    fn whois(&self, network: &str, nick: &str) -> Event {
        self.borrow().whois(network, nick)
    }

    /// Send a names request and wait for an event that answers this request (blocking)
    fn names(&self, network: &str, channel: &str) -> Event {
        self.borrow().names(network, channel)
    }

    /// Send a reply in response to some event
    fn reply(&self, event: &Event, message: &str, highlight: bool) -> Future<Response> {
        self.borrow().reply(event, message, highlight)
    }

    /// Send a reply (as a notice) in response to some event
    fn reply_with_notice(&self, event: &Event, message: &str) -> Future<Response> {
        self.borrow().reply_with_notice(event, message)
    }

    /// Send a reply (as a ctcp action) in response to some event
    fn reply_with_action(&self, event: &Event, message: &str) -> Future<Response> {
        self.borrow().reply_with_action(event, message)
    }
}

fn targets_for_event(event: &Event) -> Option<(&str, &str, &str)> {
    let params = &event.params;
    match event.event {
        EventType::Join
        | EventType::Part
        | EventType::PrivMsg
        | EventType::Notice
        | EventType::Ctcp
        | EventType::Action => Some((&params[0][..], &params[2][..], &params[1][..])),
        _ => None,
    }
}

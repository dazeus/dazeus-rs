//! DaZeus IRC bot bindings for rust
#![feature(io, std_misc, net, collections)]

extern crate "rustc-serialize" as serialize;
extern crate unix_socket;


pub use self::event::*;
pub use self::response::*;
pub use self::request::*;
pub use self::scope::*;
use self::handlers::{Reader, Writer, Handler};
use self::util::Connection;
use std::collections::HashMap;
use std::io::{Read, Write, BufStream};
use std::sync::Future;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::cell::RefCell;


mod event;
mod handlers;
mod request;
mod response;
mod scope;
pub mod error;
pub mod util;


/// Methods that need to be implemented for sending commands to the server
pub trait Commander {
    fn subscribe<F>(&self, event: EventType, callback: F) -> Future<Response> where F: Fn(Event);
    fn subscribe_command<F>(&self, command: &str, callback: F) -> Future<Response> where F: Fn(Event);
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
    fn listen(&self);
}

/// More useful methods that can be used by a commander
pub trait CommanderExt {
    fn reply(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn names(&self, network: &str, channel: &str) -> Future<Event>;
    fn whois(&self, network: &str, nick: &str) -> Future<Event>;
}

/// The base DaZeus struct
pub struct DaZeus<'a> {
    event_rx: Receiver<Event>,
    request_tx: Sender<(Request, Sender<Response>)>,
    listeners: RefCell<HashMap<EventType, Vec<Box<Fn(Event) + 'a>>>>,
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

        DaZeus { event_rx: event_rx, request_tx: request_tx, listeners: RefCell::new(HashMap::new()) }
    }

    /// Send a new Json packet to DaZeus
    pub fn send(&self, data: Request) -> Future<Response> {
        let (response_tx, response_rx) = channel();
        match self.request_tx.send((data, response_tx)) {
            Err(e) => panic!(e),
            Ok(_) => (),
        }
        Future::from_receiver(response_rx)
    }

    /// Handle an event received
    fn handle_event(&self, event: Event) {
        if self.listeners.borrow().contains_key(&event.event) {
            let liststack = self.listeners.borrow();
            let listeners = liststack.get(&event.event).unwrap();
            for listener in listeners {
                (**listener)(event.clone());
            }
        }
    }
}

impl<'a> Commander for DaZeus<'a> {
    /// Loop wait for messages to receive in a blocking way
    fn listen(&self) {
        loop {
            match self.event_rx.recv() {
                Ok(event) => self.handle_event(event),
                Err(err) => panic!(err)
            }
        }
    }

    /// Subscribe to an event type and call the callback function every time such an event occurs
    fn subscribe<F>(&self, event: EventType, callback: F) -> Future<Response>
        where F: Fn(Event) + 'a
    {
        let request = match event {
            EventType::Command(ref cmd) => Request::SubscribeCommand(cmd.clone(), None),
            _ => Request::Subscribe(event.to_string()),
        };

        if !self.listeners.borrow().contains_key(&event) {
            self.listeners.borrow_mut().insert(event.clone(), Vec::new());
        }

        // borrow listeners only for adding the listener callback
        {
            let mut liststack = self.listeners.borrow_mut();
            let mut listeners = liststack.get_mut(&event).unwrap();
            listeners.push(Box::new(callback));
        }

        // because we can't have a mutable borrow when borrowing immutable over here
        self.send(request)
    }

    /// Subscribe to a command and call the callback function every time such a command occurs
    fn subscribe_command<F>(&self, command: &str, callback: F) -> Future<Response>
        where F: Fn(Event) + 'a
    {
        self.subscribe(EventType::Command(String::from_str(command)), callback)
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
}

impl<'a> Commander for RefCell<DaZeus<'a>> {
    /// Loop wait for messages to receive in a blocking way
    fn listen(&self) {
        self.borrow().listen()
    }

    /// Subscribe to an event type and call the callback function every time such an event occurs
    fn subscribe<F>(&self, event: EventType, callback: F) -> Future<Response>
        where F: Fn(Event) + 'a
    {
        self.borrow().subscribe(event, callback)
    }

    /// Subscribe to a command and call the callback function every time such a command occurs
    fn subscribe_command<F>(&self, command: &str, callback: F) -> Future<Response>
        where F: Fn(Event) + 'a
    {
        self.borrow().subscribe_command(command, callback)
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
}

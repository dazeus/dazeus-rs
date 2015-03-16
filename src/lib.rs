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


mod event;
mod handlers;
mod request;
mod response;
mod scope;
pub mod error;
pub mod util;


/// Methods that need to be implemented for sending commands to the server
trait Commander {
    fn subscribe(&mut self, events: &str, callback: &Fn(Event)) -> Future<Response>;
    fn subscribe_command(&mut self, command: &str, callback: &Fn(Event)) -> Future<Response>;
    fn networks(&self) -> Future<Response>;
    fn channels(&self, network: &str) -> Future<Response>;
    fn message(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn notice(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn ctcp(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn ctcp_reply(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn action(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn reply(&self, network: &str, channel: &str, message: &str) -> Future<Response>;
    fn send_names(&self, network: &str, channel: &str) -> Future<Response>;
    fn names(&self, network: &str, channel: &str) -> Future<Event>;
    fn send_whois(&self, network: &str, nick: &str) -> Future<Response>;
    fn whois(&self, network: &str, nick: &str) -> Future<Event>;
    fn join(&self, network: &str, channel: &str) -> Future<Response>;
    fn part(&self, network: &str, channel: &str) -> Future<Response>;
    fn nick(&self, network: &str) -> Future<Response>;
    fn handshake(&self, name: &str, version: &str, config: Option<&str>) -> Future<Response>;
    fn get_config(&self, group: &str, name: &str) -> Future<Response>;
    fn get_property(&self, name: &str, scope: Scope) -> Future<Response>;
    fn set_property(&self, name: &str, value: &str, scope: Scope) -> Future<Response>;
    fn unset_property(&self, name: &str, name: &str, scope: Scope) -> Future<Response>;
    fn get_property_keys(&self, prefix: &str, scope: Scope) -> Future<Response>;
    fn set_permission(&self, permission: &str, allow: bool, scope: Scope) -> Future<Response>;
    fn has_permission(&self, permission: &str, default: bool, scope: Scope) -> Future<Response>;
    fn unset_permission(&self, permission: &str, scope: Scope) -> Future<Response>;
}

/// The base DaZeus struct
pub struct DaZeus<'a> {
    event_rx: Receiver<Event>,
    request_tx: Sender<(Request, Sender<Response>)>,
    listeners: HashMap<EventType, Vec<&'a Fn(Event)>>,
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

        DaZeus { event_rx: event_rx, request_tx: request_tx, listeners: HashMap::new() }
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
        if self.listeners.contains_key(&event.event) {
            let listeners = self.listeners.get(&event.event).unwrap();
            for listener in listeners {
                (**listener)(event.clone());
            }
        }
    }

    /// Loop wait for messages to receive in a blocking way
    pub fn listen(&self) {
        loop {
            match self.event_rx.recv() {
                Ok(event) => self.handle_event(event),
                Err(err) => panic!(err)
            }
        }
    }
}

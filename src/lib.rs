//! DaZeus IRC bot bindings for rust
#![feature(io, std_misc, net, collections)]

extern crate "rustc-serialize" as serialize;
extern crate unix_socket;

pub use scope::Scope;
use connection::{Reader, Writer};
use serialize::json::Json;
use std::io::{Read, Write, BufStream};
use std::sync::Future;
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use std::thread;
use util::Connection;

mod connection;
mod scope;
pub mod util;

/// The result from a command send to the DaZeus server
pub struct Result;

/// An event received from the DaZeus server
pub struct Event;

/// Methods that need to be implemented for sending commands to the server
trait Commander {
    fn subscribe(&self, events: &str, callback: Fn(Event)) -> Future<Result>;
    fn subscribe_command(&self, command: &str, callback: Fn(Event)) -> Future<Result>;
    fn networks(&self) -> Future<Result>;
    fn channels(&self, network: &str) -> Future<Result>;
    fn message(&self, network: &str, channel: &str, message: &str) -> Future<Result>;
    fn notice(&self, network: &str, channel: &str, message: &str) -> Future<Result>;
    fn ctcp(&self, network: &str, channel: &str, message: &str) -> Future<Result>;
    fn ctcp_reply(&self, network: &str, channel: &str, message: &str) -> Future<Result>;
    fn action(&self, network: &str, channel: &str, message: &str) -> Future<Result>;
    fn reply(&self, network: &str, channel: &str, message: &str) -> Future<Result>;
    fn send_names(&self, network: &str, channel: &str) -> Future<Result>;
    fn names(&self, network: &str, channel: &str) -> Future<Event>;
    fn send_whois(&self, network: &str, nick: &str) -> Future<Result>;
    fn whois(&self, network: &str, nick: &str) -> Future<Event>;
    fn join(&self, network: &str, channel: &str) -> Future<Result>;
    fn part(&self, network: &str, channel: &str) -> Future<Result>;
    fn nick(&self, network: &str) -> Future<Result>;
    fn handshake(&self, name: &str, version: &str, config: Option<&str>) -> Future<Result>;
    fn get_config(&self, group: &str, name: &str) -> Future<Result>;
    fn get_property(&self, name: &str, scope: Scope) -> Future<Result>;
    fn set_property(&self, name: &str, value: &str, scope: Scope) -> Future<Result>;
    fn unset_property(&self, name: &str, name: &str, scope: Scope) -> Future<Result>;
    fn get_property_keys(&self, prefix: &str, scope: Scope) -> Future<Result>;
    fn set_permission(&self, permission: &str, allow: bool, scope: Scope) -> Future<Result>;
    fn has_permission(&self, permission: &str, default: bool, scope: Scope) -> Future<Result>;
    fn unset_permission(&self, permission: &str, scope: Scope) -> Future<Result>;
}

/// The base DaZeus struct
pub struct DaZeus {
    read: Receiver<Json>,
    write: Sender<Json>
}

impl DaZeus {
    /// Create a new instance of DaZeus from the given connection
    pub fn from_conn(conn: Connection) -> DaZeus {
        let clone = conn.try_clone().unwrap();
        DaZeus::new(conn, clone)
    }

    /// Create a new instance of DaZeus from the given connection, making use of a buffered stream
    pub fn from_conn_buff(conn: Connection) -> DaZeus {
        let clone = conn.try_clone().unwrap();
        DaZeus::new(BufStream::new(conn), BufStream::new(clone))
    }

    /// Create a new instance from a Read and Send, note that both need to point to the same socket
    pub fn new<R: Read + Send + 'static, W: Write + Send + 'static>(read: R, write: W) -> DaZeus {
        let (read_tx, read_rx) = channel();
        let (write_tx, write_rx) = channel();
        let mut reader = Reader::new(read, read_tx);
        let mut writer = Writer::new(write, write_rx);

        thread::spawn(move || { reader.run(); });
        thread::spawn(move || { writer.run(); });

        DaZeus { read: read_rx, write: write_tx }
    }

    /// Send a new Json packet to DaZeus
    pub fn send(&self, data: Json) {
        self.write.send(data).unwrap();
    }

    /// Send a string that must be valid Json to DaZeus
    pub fn send_json_str(&self, data: &str) {
        let json = Json::from_str(data).unwrap();
        self.send(json);
    }

    /// Check if there are any new messages available
    /// Will not block and immidiately either return None if there is no message available.
    pub fn try_receive(&self) -> Option<Json> {
        match self.read.try_recv() {
            Ok(val) => Some(val),
            Err(TryRecvError::Empty) => None,
            _ => panic!("Channel was unexpectedly closed")
        }
    }

    /// Blockingly check for any new messages
    pub fn receive(&self) -> Json {
        match self.read.recv() {
            Ok(val) => val,
            _ => panic!("Channel was unexpectedly closed")
        }
    }

    /// Loop wait for messages to receive in a blocking way
    pub fn listen(&self) {
        loop {
            let _ = self.receive();
        }
    }
}

//! DaZeus IRC bot bindings for rust
#![feature(io, std_misc, net, collections)]

extern crate "rustc-serialize" as serialize;
extern crate unix_socket;

use std::io::{Read, Write};
use std::sync::Future;
use serialize::json::Json;
use std::str::from_utf8;
pub use scope::Scope;

pub mod util;
mod scope;

/// The result from a command send to the DaZeus server
pub struct Result;

/// An event received from the DaZeus server
pub struct Event;

/// The base DaZeus object
pub struct DaZeus<T: Read + Write> {
    connection: T,
    buffer: Vec<u8>,
    message_len: usize,
    offset: usize
}

/// Methods that need to be implemented for basic listening services
trait Listener {
    fn subscribe(&self, events: &str, callback: Fn(Event)) -> Future<Result>;
    fn subscribe_command(&self, command: &str, callback: Fn(Event)) -> Future<Result>;
}

/// Methods that need to be implemented for sending commands to the server
trait Commander {
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

impl<T: Read + Write> DaZeus<T> {
    pub fn new(connection: T) -> DaZeus<T> {
        DaZeus {
            connection: connection,
            buffer: Vec::new(),
            message_len: 0,
            offset: 0
        }
    }

    pub fn send(&mut self, obj: Json) {
        let encoded = obj.to_string();
        self.send_str(&encoded);
    }

    pub fn send_str(&mut self, data: &str) {
        let len = data.bytes().len();
        let message = format!("{}{}", len, data);
        match self.connection.write_all(message.as_bytes()) {
            Err(e) => panic!(e),
            _ => match self.connection.flush() {
                Err(e) => panic!(e),
                _ => ()
            }
        }
    }

    pub fn listen(&mut self) {
        loop {
            // step one: see if there is some new data on the socket
            self.retrieve_from_socket();

            // loop this part as well, because we may have more messages
            loop {
                // step two: find message length at the start of the buffer
                self.find_message_len();

                // step three: find a message
                if !self.try_process_message() {
                    break
                }
            }
        }
    }

    /// Retrieve new data from the socket
    fn retrieve_from_socket(&mut self) {
        let mut buf = [0; 1024];
        match self.connection.read(&mut buf) {
            Ok(bytes) => {
                self.buffer.push_all(&buf[..bytes]);
            },
            Err(e) => panic!(e),
        }
    }

    /// Find where a message is located
    fn find_message_len(&mut self) {
        while self.offset < self.buffer.len() {
            // check for a number
            if self.buffer[self.offset] < 0x40 && self.buffer[self.offset] >= 0x30 {
                self.message_len *= 10;
                self.message_len += (self.buffer[self.offset] - 0x30) as usize;
                self.offset += 1;

            // skip newline and carriage return
            } else if self.buffer[self.offset] == 0xa || self.buffer[self.offset] == 0xd {
                self.offset += 1;
            } else {
                break;
            }
        }
    }

    /// Try to process a message on the buffer.
    /// Return false if none could be processed or true if one was processed.
    fn try_process_message(&mut self) -> bool {
        if self.message_len > 0 && self.buffer.len() >= self.message_len + self.offset {
            self.handle_message();

            // remove bytes from buffer and reset
            self.buffer = self.buffer.split_off(self.message_len + self.offset);
            self.message_len = 0;
            self.offset = 0;
            true
        } else {
            false
        }
    }

    /// A message was found on the buffer, handle parsing and send it off
    fn handle_message(&mut self) {
        let end = self.offset + self.message_len;
        let message = &self.buffer[self.offset..end];

        match from_utf8(message) {
            Ok(s) => {
                println!("{}", s);
            },
            Err(e) => panic!(e)
        }
    }
}

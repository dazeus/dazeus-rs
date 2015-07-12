use std::io::{Read, Write};
use serialize::json::{ToJson, Json};
use std::str::{from_utf8};
use super::response::Response;
use super::event::{Event, is_event_json};
use super::request::Request;
use super::error::Error;
use std::borrow::ToOwned;

pub enum Message {
    Response(Response),
    Event(Event),
}

pub struct Handler<T> {
    socket: T,
    buffer: Vec<u8>,
}

impl<T> Handler<T> where T: Read + Write {
    pub fn new(socket: T) -> Handler<T> {
        Handler { socket: socket, buffer: Vec::new() }
    }

    pub fn read(&mut self) -> Result<Message, Error> {
        loop {
            if let Some((offset, len)) = self.find_message() {
                return self.make_message(offset, len);
            }

            try!(self.retrieve_from_socket());
        }
    }

    /// Retrieve new data from the socket
    fn retrieve_from_socket(&mut self) -> Result<(), Error> {
        let mut buf = [0; 1024];
        let bytes = try!(self.socket.read(&mut buf));
        for b in buf[..bytes].iter() {
            self.buffer.push(*b);
        }
        Ok(())
    }

    /// Find where a message is located
    fn find_message(&self) -> Option<(usize, usize)> {
        let mut offset = 0;
        let mut message_len = 0;

        while offset < self.buffer.len() {
            // check for a number
            if self.buffer[offset] < 0x3A && self.buffer[offset] >= 0x30 {
                message_len *= 10;
                message_len += (self.buffer[offset] - 0x30) as usize;
                offset += 1;

            // skip newline and carriage return
            } else if self.buffer[offset] == 0xa || self.buffer[offset] == 0xd {
                offset += 1;
            } else {
                break;
            }
        }

        if message_len > 0 && self.buffer.len() >= offset + message_len {
            debug!("Found message in buffer starting at {} with length {}", offset, message_len);
            Some((offset, message_len))
        } else {
            debug!("Found no complete message in buffer");
            None
        }
    }

    fn make_message(&mut self, offset: usize, length: usize) -> Result<Message, Error> {
        let end = offset + length;
        assert!(self.buffer.len() >= end);

        // check the result of our conversion
        let json_try = match from_utf8(&self.buffer[offset..end]) {
            Ok(json_str) => Ok(Json::from_str(json_str)),
            Err(e) => Err(e),
        };

        // first make sure we have a correct internal state
        self.buffer = self.buffer[offset+length..].to_owned(); // iter().collect();

        let json = try!(try!(json_try));

        if is_event_json(&json) {
            let evt = try!(Event::from_json(&json));
            debug!("Valid event received: {}", json);
            Ok(Message::Event(evt))
        } else {
            let resp = try!(Response::from_json(&json));
            debug!("Valid response received: {}", json);
            Ok(Message::Response(resp))
        }
    }

    pub fn write(&mut self, request: Request) -> Result<(), Error> {
        let encoded = request.to_json().to_string();
        debug!("Sending message: {}", encoded);

        let bytes = encoded.as_bytes();
        try!(self.socket.write_all(format!("{}", bytes.len()).as_bytes()));
        try!(self.socket.write_all(bytes));
        Ok(())
    }
}

use std::io::{Read, Write};
use std::sync::mpsc::{Sender, Receiver};
use serialize::json::{ToJson, Json};
use std::str::{from_utf8, Utf8Error};
use super::response::Response;
use super::event::{Event, is_event_json};
use super::request::Request;
use std::collections::VecDeque;
use std::io::Error as IoError;
use serialize::json::ParserError as JsonParserError;
use super::error::Error;

pub enum Message {
    Response(Response),
    Event(Event),
}

pub struct Handler<T> where T: Read + Write {
    socket: T,
    buffer: Vec<u8>,
}

impl<T> Handler<T> where T: Read + Write {
    fn new(socket: T) -> Handler<T> {
        Handler { socket: socket, buffer: Vec::new() }
    }

    pub fn read(&mut self) -> Result<Message, Error> {
        loop {
            self.retrieve_from_socket();
            if let Some((offset, len)) = self.find_message() {
                debug!("Found message if buffer starting at {} with length {}", offset, len);
                return self.make_message(offset, len);
            }
        }
    }

    /// Retrieve new data from the socket
    fn retrieve_from_socket(&mut self) -> Result<(), Error> {
        let mut buf = [0; 1024];
        let bytes = try!(self.socket.read(&mut buf));
        self.buffer.push_all(&buf[..bytes]);
    }

    /// Find where a message is located
    fn find_message(&self) -> Option<(usize, usize)> {
        let offset = 0;
        let message_len = 0;

        while offset < self.buffer.len() {
            // check for a number
            if self.buffer[offset] < 0x40 && self.buffer[offset] >= 0x30 {
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
            Some((offset, message_len))
        } else {
            None
        }
    }

    fn make_message(&mut self, offset: usize, length: usize) -> Result<Message, Error> {
        let end = offset + length;
        assert!(self.buffer.len() >= end);

        // create a copy of the slice
        let conversion_result = from_utf8(&self.buffer[offset..end]);

        // first make sure we have a correct internal state
        self.buffer = self.buffer.split_off(offset + length);

        // check the result of our conversion
        let json_str = try!(conversion_result);
        let json = try!(Json::from_str(json_str));

        if is_event_json(&json) {
            Ok(Message::Event(try!(Event::from_json(&json))))
        } else {
            Ok(Message::Response(try!(Response::from_json(&json))))
        }
    }

    pub fn write(&mut self, request: Request) -> Result<(), Error> {
        let encoded = request.to_json().to_string();
        debug!("Sending message: {}", encoded);

        let bytes = encoded.as_bytes();
        try!(self.socket.write_all(format!("{}", bytes.len()).as_bytes()));
        try!(self.socket.write_all(bytes));
    }
}

// pub struct Handler {
//     write_to: Sender<Json>,
//     event_to: Sender<Event>,
//     response_to: VecDeque<Sender<Response>>,
// }
//
// impl Handler {
//     pub fn new(write_to: Sender<Json>, event_to: Sender<Event>) -> Handler {
//         Handler {
//             write_to: write_to,
//             event_to: event_to,
//             response_to: VecDeque::new()
//         }
//     }
//
//     pub fn run(&mut self, read_from: Receiver<Json>, request_from: Receiver<(Request, Sender<Response>)>) {
//         loop {
//             select!(
//                 data = read_from.recv() => {
//                     if !self.handle_socket_msg(data.unwrap()) {
//                         break;
//                     }
//                 },
//                 req = request_from.recv() => {
//                     let (request, respond_to) = req.unwrap();
//                     if !self.handle_request(request, respond_to) {
//                         break;
//                     }
//                 }
//             )
//         }
//     }
//
//     fn handle_socket_msg(&mut self, data: Json) -> bool {
//         if is_event_json(&data) {
//             self.handle_event(data)
//         } else {
//             self.handle_response(data)
//         }
//     }
//
//     fn handle_event(&self, data: Json) -> bool {
//         match Event::from_json(&data) {
//             Ok(evt) => match self.event_to.send(evt) {
//                 Ok(_) => true,
//                 Err(e) => panic!("Could not send event back to controller: {}", e),
//             },
//             Err(e) => panic!("Got an invalid json event: {}", e),
//         }
//     }
//
//     fn handle_response(&mut self, data: Json) -> bool {
//         match Response::from_json(data) {
//             Ok(resp) => match self.response_to.pop_front() {
//                 Some(sender) => match sender.send(resp) {
//                     Ok(_) => true,
//                     // We'll just ignore this response if the future containing the response
//                     // channel was discarded.
//                     Err(_) => true,
//                 },
//                 None => panic!("Got a response, but there is nothing to repond to"),
//             },
//             Err(e) => panic!("Got an invalid json response: {}", e),
//         }
//     }
//
//     fn handle_request(&mut self, request: Request, respond_to: Sender<Response>) -> bool {
//         let json = request.to_json();
//         match self.write_to.send(json) {
//             Ok(_) => {
//                 self.response_to.push_back(respond_to);
//                 true
//             },
//             Err(e) => panic!("Could not send message to be written on the socket: {}", e),
//         }
//     }
// }

use std::io::{Read, Write};
use std::sync::mpsc::{Sender, Receiver};
use serialize::json::{ToJson, Json};
use std::str::from_utf8;
use super::response::Response;
use super::event::{Event, is_event_json};
use super::request::Request;
use std::collections::VecDeque;

pub struct Reader<T: Read> {
    socket: T,
    actions: Sender<Json>,
    buffer: Vec<u8>,
    message_len: usize,
    offset: usize
}

impl<T: Read> Reader<T> {
    pub fn new(socket: T, actions: Sender<Json>) -> Reader<T> {
        Reader { socket: socket, actions: actions, buffer: Vec::new(), message_len: 0, offset: 0 }
    }

    pub fn run(&mut self) {
        loop {
            self.retrieve_from_socket();
            loop {
                self.find_message_len();
                if !self.try_process_message() {
                    break
                }
            }
        }
    }

    /// Retrieve new data from the socket
    fn retrieve_from_socket(&mut self) {
        let mut buf = [0; 1024];
        match self.socket.read(&mut buf) {
            Ok(bytes) => {
                self.buffer.push_all(&buf[..bytes]);
            },
            Err(e) => panic!("Could not retrieve bytes from socket: {}", e),
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
                debug!("Received message: {}", s);
                let res = Json::from_str(s).unwrap();
                match self.actions.send(res) {
                    Err(e) => panic!("Could not send received message: {}", e),
                    _ => ()
                }
            },
            Err(e) => panic!("Could not convert received message to utf-8: {}", e)
        }
    }
}

pub struct Writer<T: Write> {
    socket: T,
    actions: Receiver<Json>
}

impl<T: Write> Writer<T> {
    pub fn new(socket: T, actions: Receiver<Json>) -> Writer<T> {
        Writer { socket: socket, actions: actions }
    }

    pub fn run(&mut self) {
        loop {
            match self.actions.recv() {
                Ok(data) => {
                    let encoded = data.to_string();
                    debug!("Sending message: {}", encoded);
                    let len = encoded.bytes().len();
                    let message = format!("{}{}", len, encoded);
                    match self.socket.write_all(message.as_bytes()) {
                        Err(e) => panic!("Could not write message to socket: {}", e),
                        _ => match self.socket.flush() {
                            Err(e) => panic!("Could not flush socket: {}", e),
                            _ => ()
                        }
                    }
                },
                _ => panic!("Channel receiving messages to be sent was unexpectedly closed")
            }
        }
    }
}

pub struct Handler {
    write_to: Sender<Json>,
    event_to: Sender<Event>,
    response_to: VecDeque<Sender<Response>>,
}

impl Handler {
    pub fn new(write_to: Sender<Json>, event_to: Sender<Event>) -> Handler {
        Handler {
            write_to: write_to,
            event_to: event_to,
            response_to: VecDeque::new()
        }
    }

    pub fn run(&mut self, read_from: Receiver<Json>, request_from: Receiver<(Request, Sender<Response>)>) {
        loop {
            select!(
                data = read_from.recv() => {
                    if !self.handle_socket_msg(data.unwrap()) {
                        break;
                    }
                },
                req = request_from.recv() => {
                    let (request, respond_to) = req.unwrap();
                    if !self.handle_request(request, respond_to) {
                        break;
                    }
                }
            )
        }
    }

    fn handle_socket_msg(&mut self, data: Json) -> bool {
        if is_event_json(&data) {
            self.handle_event(data)
        } else {
            self.handle_response(data)
        }
    }

    fn handle_event(&self, data: Json) -> bool {
        match Event::from_json(&data) {
            Ok(evt) => match self.event_to.send(evt) {
                Ok(_) => true,
                Err(e) => panic!("Could not send event back to controller: {}", e),
            },
            Err(e) => panic!("Got an invalid json event: {}", e),
        }
    }

    fn handle_response(&mut self, data: Json) -> bool {
        match Response::from_json(data) {
            Ok(resp) => match self.response_to.pop_front() {
                Some(sender) => match sender.send(resp) {
                    Ok(_) => true,
                    // We'll just ignore this response if the future containing the response
                    // channel was discarded.
                    Err(_) => true,
                },
                None => panic!("Got a response, but there is nothing to repond to"),
            },
            Err(e) => panic!("Got an invalid json response: {}", e),
        }
    }

    fn handle_request(&mut self, request: Request, respond_to: Sender<Response>) -> bool {
        let json = request.to_json();
        match self.write_to.send(json) {
            Ok(_) => {
                self.response_to.push_back(respond_to);
                true
            },
            Err(e) => panic!("Could not send message to be written on the socket: {}", e),
        }
    }
}

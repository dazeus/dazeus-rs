use std::io::{Read, Write};
use std::sync::mpsc::{Sender, Receiver};
use serialize::json::Json;
use std::str::from_utf8;

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
                let res = Json::from_str(s).unwrap();
                match self.actions.send(res) {
                    Err(e) => panic!(e),
                    _ => ()
                }
            },
            Err(e) => panic!(e)
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
                    let len = encoded.bytes().len();
                    let message = format!("{}{}", len, encoded);
                    match self.socket.write_all(message.as_bytes()) {
                        Err(e) => panic!(e),
                        _ => match self.socket.flush() {
                            Err(e) => panic!(e),
                            _ => ()
                        }
                    }
                },
                _ => panic!("Channel was unexpectedly closed")
            }
        }
    }
}

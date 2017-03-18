use super::types::Message;

use std::sync::mpsc::{channel, Sender};
use std::{thread};
use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::collections::{VecDeque};
use rustc_serialize::json;

const TCP_BUFFER_SIZE: usize = 1000000;

pub struct Listener {
    port: u32,
    core_link: Sender<Message>,
}

impl Listener {
    pub fn new(port: u32, link: Sender<Message>) -> Listener {
        Listener {
            port: port,
            core_link: link,
        }
    }

    fn handle_connection(mut stream: TcpStream, link: Sender<Message>) {
        let mut buffer: [u8; TCP_BUFFER_SIZE] = [0; TCP_BUFFER_SIZE];
        let mut parser = MessageParser::new();

        loop {
            if let Ok(bytes_read) = stream.read(&mut buffer) {
                let slice = &buffer[0..bytes_read];
                parser.push(slice);

                while let Some(msg) = parser.pop() {
                    match link.send(msg) {
                        Ok(_)  => {},
                        Err(_) => println!("(Listener) Failed to send a message to main thread"),
                    };
                }
            }
        }
    }

    fn listen_to_clients(port: u32, link: Sender<Message>) {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(addr.as_str()).unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("(Connection) New client, {:?}!", stream);

                    let link = link.clone();

                    thread::spawn(move || {
                        Listener::handle_connection(stream, link);
                    });
                },
                Err(_) => {},
            }
        }
    }

    pub fn run(&self) {
        loop {
            println!("(Listener) Listening on port {}", self.port);

            let (connections_in, connections_out) = channel::<Message>();
            let port = self.port;

            thread::spawn(move || {
                Listener::listen_to_clients(port, connections_in);
            });

            loop {
                match connections_out.try_recv() {
                    Ok(msg) => {
                        match self.core_link.send(msg) {
                            Ok(_)  => {},
                            Err(_) => println!("(Listener) Failed to send a message to core"),
                        };
                    },
                    Err(_)  => {},
                };
            }
        }
    }
}

struct MessageParser {
    buffer: Vec<u8>,
    open_parentheses: u32,
    parentheses_counter: u32,
    messages: VecDeque<Message>,
}

impl MessageParser {
    fn new() -> MessageParser {
        MessageParser {
            buffer: vec![],
            open_parentheses: 0,
            parentheses_counter: 0,
            messages: VecDeque::<Message>::new(),
        }
    }

    fn parse(text: &[u8]) -> Option<Message> {
        let message_utf8 = match String::from_utf8(text.to_vec()) {
            Ok(utf) => utf,
            Err(_)  => {
                println!("(Parser) Message is not utf8-encoded");
                return None;
            },
        };

        match json::decode(&message_utf8) {
            Ok(msg) => Some(msg),
            Err(_)  => {
                println!("(Parser) Invalid JSON object");
                None
            },
        }
    }

    fn push(&mut self, text: &[u8]) {
        let len_prev = self.buffer.len();
        self.buffer.extend(text);

        let mut msg_start: usize = 0;

        for i in len_prev..self.buffer.len() {
            match self.buffer[i] as char {
                '{' => {
                    self.open_parentheses += 1;
                    self.parentheses_counter += 1;
                },
                '}' => {
                    if self.open_parentheses != 0 {
                        self.open_parentheses -= 1;
                    }
                },
                _   => {},
            };

            if self.open_parentheses == 0 {
                let slice = &self.buffer[msg_start..(i+1)];
                msg_start = i + 1;

                if self.parentheses_counter > 0 {
                    if let Some(msg) = MessageParser::parse(slice) {
                        self.messages.push_back(msg);
                    }
                }

                self.parentheses_counter = 0;
            }
        }

        if msg_start != 0 {
            self.buffer = self.buffer.split_off(msg_start);
        }
    }

    fn pop(&mut self) -> Option<Message> {
        self.messages.pop_front()
    }
}

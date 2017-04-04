use types::message::{MessageIn, MessageOut};
use types::double_channel::{channel, Endpoint};

use std::{thread};
use std::net::{TcpListener, TcpStream, Ipv4Addr};
use std::io::{Write, Read};
use std::time::Duration;
use std::collections::{VecDeque, HashMap};
use rustc_serialize::json;

type ConnectionLink = Endpoint<MessageOut, Option<MessageIn>>;
type ConnectionData = (String, ConnectionLink);

const TCP_BUFFER_SIZE: usize = 1000000;

pub struct Listener {
    port: u32,
    address: Ipv4Addr,
    link_core: Endpoint<MessageIn, MessageOut>,
}

impl Listener {
    pub fn new(address: Ipv4Addr, port: u32, link: Endpoint<MessageIn, MessageOut>) -> Listener {
        Listener {
            port: port,
            address: address,
            link_core: link,
        }
    }

    fn handle_connection(mut stream: TcpStream, link: Endpoint<Option<MessageIn>, MessageOut>) {
        let mut buffer: [u8; TCP_BUFFER_SIZE] = [0; TCP_BUFFER_SIZE];
        let mut parser = MessageParser::new();

        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    let slice = &buffer[0..bytes_read];
                    parser.push(slice);

                    while let Some(msg) = parser.pop() {
                        match link.send(Some(msg)) {
                            Ok(_)  => {},
                            Err(_) => println!("(Connection) Failed to send a message to main thread"),
                        };
                    }
                },
                Err(_) => { },
            }

            if let Ok(response) = link.try_recv() {
                let json_response = format!("{}\n", json::as_json(&response).to_string());
                let _ = stream.write(json_response.as_bytes());
            }
        }
    }

    fn listen_to_clients(address: Ipv4Addr, port: u32, link: Endpoint<ConnectionData, ()>) {
        let listener = TcpListener::bind(format!("{}:{}", address, port))
            .expect("Invalid IP address or port");

        let five_milliseconds = Duration::from_millis(20);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("(Connection) New client, {:?}!", stream);

                    let _ = stream.set_read_timeout(Some(five_milliseconds));
                    let _ = stream.set_nodelay(true);

                    let connection_name = match stream.peer_addr() {
                        Ok(addr) => format!("{:?}:{}", addr.ip(), addr.port()),
                        Err(_)   => {
                            println!("(Listener) Could not determine client address");
                            continue;
                        }
                    };

                    let (ch_connection, ch_me_connection) = channel::<Option<MessageIn>, MessageOut>();

                    thread::spawn(move || {
                        Listener::handle_connection(stream, ch_connection);
                    });

                    let _ = link.send((connection_name, ch_me_connection));
                },
                Err(_) => {},
            }
        }
    }

    pub fn run(&self) {
        loop {
            println!("(Listener) Listening on port {}", self.port);

            let address = self.address;
            let port = self.port;

            let mut connections = HashMap::<String, ConnectionLink>::new();
            let mut publishers = HashMap::<String, String>::new();

            let (ch_listener, ch_me_listener) = channel::<ConnectionData, ()>();

            thread::spawn(move || {
                Listener::listen_to_clients(address, port, ch_listener);
            });

            loop {
                if let Ok((name, link)) = ch_me_listener.try_recv() {
                    let _ = connections.insert(name, link);
                }

                let mut closed_connections: Vec<String> = vec![];
                for (name, link) in &connections {
                    if let Ok(option_msg) = link.try_recv() {
                        match option_msg {
                            Some(msg) => {
                                let _ = publishers.insert(msg.publisher.clone(), name.clone());
                                let _ = self.link_core.send(msg);
                            },
                            None => {
                                closed_connections.push(name.clone());
                            }
                        }
                    };
                }
                for name in closed_connections {
                    connections.remove(&name);
                }

                let mut removed_publishers: Vec<String> = vec![];
                if let Ok(msg) = self.link_core.try_recv() {
                    match publishers.get(&msg.publisher) {
                        Some(name) => if let Some(link) = connections.get(name) {
                                let _ = link.send(msg);
                            } else {
                                removed_publishers.push(String::from(msg.publisher));
                            },
                        None => {
                            println!("(Networking) Publisher {} not found", &msg.publisher);
                        }
                    }
                }
                for publisher in removed_publishers {
                    publishers.remove(&publisher);
                }
            }
        }
    }
}

struct MessageParser {
    buffer: Vec<u8>,
    open_parentheses: u32,
    parentheses_counter: u32,
    messages: VecDeque<MessageIn>,
}

impl MessageParser {
    fn new() -> MessageParser {
        MessageParser {
            buffer: vec![],
            open_parentheses: 0,
            parentheses_counter: 0,
            messages: VecDeque::<MessageIn>::new(),
        }
    }

    fn parse(text: &[u8]) -> Option<MessageIn> {
        let message_utf8 = match String::from_utf8(text.to_vec()) {
            Ok(utf) => utf,
            Err(_)  => {
                println!("(Parser) Message is not utf8-encoded");
                return None;
            },
        };

        let decoded: Result<MessageIn, _> = json::decode(&message_utf8);
        match decoded {
            Err(_)  => {
                println!("(Parser) Invalid JSON object");
                None
            },
            Ok(msg) => {
                Some(msg)
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

    fn pop(&mut self) -> Option<MessageIn> {
        self.messages.pop_front()
    }
}

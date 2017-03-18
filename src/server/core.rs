use super::types::Message;
use super::networking::Listener;
use super::console::Console;

use std::sync::mpsc::channel;
use std::thread;
use rustc_serialize::json;

pub struct Server {
    port: u32,
}

impl Server {
    pub fn new(port: u32) -> Server {
        Server {
            port: port,
        }
    }

    pub fn run(&self) {
        let (listener_in, listener_out) = channel::<Message>();
        let (console_in, console_out) = channel::<String>();

        {
            let p = self.port;
            thread::spawn(move || {
                let listener = Listener::new(p, listener_in);
                listener.run();
            });

            thread::spawn(move || {
                let console = Console::new(console_in);
                console.run();
            });
        }

        loop {
            if let Ok(msg) = listener_out.try_recv() {
                println!("(Core) object received:\n {}", json::as_pretty_json(&msg));
            }

            if let Ok(command) = console_out.try_recv() {
                println!("(Core) console input: \"{}\"", command);
            }
        }
    }
}

use types::message::{MessageIn, MessageOut};
use types::double_channel::{channel, Endpoint};
use server::networking::Listener;
use server::console::Console;
use visualization::core::Visualization;

use std::thread;
use std::collections::HashMap;
use std::fmt::Write;
use std::net;
use std::time::Duration;

pub struct Server {
    port: u32,
    address: net::Ipv4Addr,
    visualizations: HashMap<String, Endpoint<Option<MessageIn>, Option<MessageOut>>>,
}

impl Server {
    pub fn new(address: net::Ipv4Addr, port: u32) -> Server {
        Server {
            port: port,
            address: address,
            visualizations: HashMap::<String, Endpoint<Option<MessageIn>, Option<MessageOut>>>::new(),
        }
    }

    pub fn run(&mut self) {
        let (ch_listener, ch_me_listener) = channel::<MessageIn, MessageOut>();
        let (ch_console, ch_me_console) = channel::<String, String>();

        {
            let port = self.port;
            let address = self.address;

            thread::spawn(move || {
                let listener = Listener::new(address, port, ch_listener);
                listener.run();
            });

            thread::spawn(move || {
                let console = Console::new(ch_console);
                console.run();
            });
        }

        loop {
            if let Ok(msg) = ch_me_listener.try_recv() {
                if let Some(link) = self.visualizations.get(&msg.publisher) {
                    let _ = link.send(Some(msg));
                }
            }

            if let Ok(command) = ch_me_console.try_recv() {
                let response = self.execute_command(command);
                let _ = ch_me_console.send(response);
            }

            let mut removed_visualizations: Vec<String> = vec![];
            for (name, link) in &self.visualizations {
                if let Ok(response) = link.try_recv() {
                    match response {
                        Some(msg) => { let _ = ch_me_listener.send(msg); },
                        None      => { removed_visualizations.push(name.clone()); },
                    };
                }
            }
            for name in removed_visualizations {
                let _ = ch_me_console.send(format!("Visualization {} has been stopped", name));
                self.stop_visualization(name);
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    fn execute_command(&mut self, command: String) -> String {
        let words: Vec<&str> = command.split(" ").collect();

        if words.len() == 0 {
            return "".to_string();
        }

        match (words[0], words.len()) {
            ("list", 1)  => self.print_visualizations(),
            ("start", 3) => self.start_visualization(words[1].to_string(), words[2].to_string()),
            ("stop", 2)  => self.stop_visualization(words[1].to_string()),
            (cmd @ _, _) => format!("unknown command: \"{}\"", cmd).to_string(),
        }
    }

    fn print_visualizations(&self) -> String {
        let mut response = "Running visualizations:".to_string();

        for (publisher, _) in &self.visualizations {
            let _ = write!(&mut response, "\n{}", publisher);
        }

        response
    }

    fn start_visualization(&mut self, publisher: String, configuration: String) -> String {
        let (ch_window, ch_me_window) = channel::<Option<MessageOut>, Option<MessageIn>>();

        let p = publisher.clone();
        thread::spawn(move || {
            let mut visualization = Visualization::new(ch_window, p, configuration);
            visualization.run();
        });

        let status = self.visualizations.insert(publisher, ch_me_window);

        let info = match status {
            Some(link) => {
                let _ = link.send(None);
                "Warning: closing previous visualization\n".to_string()
            }
            None       => {
                String::from("")
            }
        };
        format!("{}New visualization started succesfully", info)
    }

    fn stop_visualization(&mut self, publisher: String) -> String {
        match self.visualizations.remove(&publisher) {
            Some(link) => {
                let _ = link.send(None);
                format!("Visualization {} stopped succesfully", publisher)
            },
            None       => format!("Visualization {} isn't currently running", publisher),
        }
    }
}

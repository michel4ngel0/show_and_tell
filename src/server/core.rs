use types::message::{MessageIn, MessageOut};
use types::double_channel::{channel, Endpoint};
use server::networking::Listener;
use server::console::Console;
use visualization::core::Visualization;

use std::thread;
use std::collections::HashMap;
use std::net;
use std::time::{Instant, Duration};
use std::fs::File;
use std::io::Write;

pub struct Server {
    port: u32,
    address: net::Ipv4Addr,
    visualizations: HashMap<String, Endpoint<Option<MessageIn>, Option<MessageOut>>>,
    traffic_log_file: Option<(File, String)>,
}

impl Server {
    pub fn new(address: net::Ipv4Addr, port: u32) -> Server {
        Server {
            port: port,
            address: address,
            visualizations: HashMap::<String, Endpoint<Option<MessageIn>, Option<MessageOut>>>::new(),
            traffic_log_file: None,
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
            let time_now = Instant::now();

            if let Ok(msg) = ch_me_listener.try_recv() {
                if let Some(link) = self.visualizations.get(&msg.publisher) {
                    let log = format!("{:?}\n", msg);
                    match self.traffic_log_file {
                        None => {},
                        Some((ref mut file, _)) => { let _ = file.write(log.as_bytes()); },
                    }
                    let _ = link.send(Some(msg));
                }
            }

            if let Ok(command) = ch_me_console.try_recv() {
                let (response, quit) = self.execute_command(command);
                let _ = ch_me_console.send(response);

                if quit {
                    thread::sleep(Duration::from_millis(50));
                    break;
                }
            }

            let mut removed_visualizations: Vec<String> = vec![];
            for (name, link) in &self.visualizations {
                if let Ok(response) = link.try_recv() {
                    match response {
                        Some(msg) => {
                            let log = format!("{:?}\n", msg);
                            match self.traffic_log_file {
                                None => {},
                                Some((ref mut file, _)) => { let _ = file.write(log.as_bytes()); },
                            }
                            let _ = ch_me_listener.send(msg);
                        },
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

    fn execute_command(&mut self, command: String) -> (String, bool) {
        let words: Vec<&str> = command.split(" ").collect();

        if words.len() == 0 {
            return ("".to_string(), false);
        }

        match (words[0], words.len()) {
            ("list", 1)   => (self.print_visualizations(), false),
            ("start", 3)  => (self.start_visualization(words[1].to_string(), words[2].to_string()), false),
            ("close", 2)  => (self.stop_visualization(words[1].to_string()), false),
            ("log", 2) |
            ("log", 3)    => (self.launch_or_stop_traffic_log(words), false),
            ("quit", 1) |
            ("exit", 1)   => (String::from("Shutting down"), true),
            (cmd @ _, _)  => (format!("Unknown command: \"{}\"", cmd).to_string(), false),
        }
    }

    fn print_visualizations(&self) -> String {
        let mut response = "Running visualizations:".to_string();

        use std::fmt::Write as FmtWrite;
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

    fn launch_or_stop_traffic_log(&mut self, args: Vec<&str>) -> String {
        use std::path::Path;

        match (args[1], args.len()) {
            ("start", 3) => {
                let filename = args[2];
                let path = Path::new(&filename);
                match File::create(path) {
                    Err(_) => {
                        format!("Failed to open {}", filename)
                    },
                    Ok(handle) => {
                        self.traffic_log_file = Some((handle, filename.to_string()));
                        format!("Saving logs to {}", filename)
                    }
                }
            }
            ("stop", 2)  => {
                let status = match self.traffic_log_file {
                    None                    => format!("No logger running"),
                    Some((_, ref mut name)) => format!("Logs saved to {}", name),
                };
                self.traffic_log_file = None;
                status
            },
            _            => format!("Invalid command: \"{}\"", args.join(" "))
        }
    }
}

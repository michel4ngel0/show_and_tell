use types::message::Message;
use server::networking::Listener;
use server::console::Console;
use visualization::core::Visualization;

use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::collections::HashMap;
use std::fmt::Write;

pub struct Server {
    port: u32,
    visualizations: HashMap<String, Sender<Option<Message>>>,
}

impl Server {
    pub fn new(port: u32) -> Server {
        Server {
            port: port,
            visualizations: HashMap::<String, Sender<Option<Message>>>::new(),
        }
    }

    pub fn run(&mut self) {
        let (listener_in, listener_out) = channel::<Message>();
        let (console_in, console_out) = channel::<String>();
        let (self_in, self_out) = channel::<String>();

        {
            let p = self.port;
            thread::spawn(move || {
                let listener = Listener::new(p, listener_in);
                listener.run();
            });

            thread::spawn(move || {
                let console = Console::new(console_in, self_out);
                console.run();
            });
        }

        loop {
            if let Ok(msg) = listener_out.try_recv() {
                if let Some(link) = self.visualizations.get(&msg.publisher) {
                    let _ = link.send(Some(msg));
                }
            }

            if let Ok(command) = console_out.try_recv() {
                let response = self.execute_command(command);
                let _ = self_in.send(response);
            }
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

    fn start_visualization(&mut self, publisher: String, pipeline: String) -> String {
        let (self_in, self_out) = channel::<Option<Message>>();

        let p = publisher.clone();
        thread::spawn(move || {
            let visualization = Visualization::new(self_out, p, pipeline);
            visualization.run();
        });

        let status = self.visualizations.insert(publisher, self_in);

        match status {
            Some(link) => {
                let _ = link.send(None);
                "Warning: closing previous visualization".to_string()
            }
            None       => "New visualization started succesfully".to_string()
        }
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

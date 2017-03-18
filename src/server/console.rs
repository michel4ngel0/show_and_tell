use std::sync::mpsc::{Sender, Receiver, channel};
use std::io;
use std::thread;

pub struct Console {
    self_in: Sender<String>,
    core_out: Receiver<String>,
}

impl Console {
    pub fn new(link_in: Sender<String>, link_out: Receiver<String>) -> Console {
        Console {
            self_in: link_in,
            core_out: link_out,
        }
    }

    fn non_blocking_stdin(link: Sender<String>) {
        let mut line = String::new();

        loop {
            if let Ok(_) = io::stdin().read_line(&mut line) {
                let trimmed = String::from(line.trim());
                let _ = link.send(trimmed);
                line.clear();
            }
        }
    }

    pub fn run(&self) {
        let (input_in, input_out) = channel::<String>();

        thread::spawn(move || {
            Console::non_blocking_stdin(input_in);
        });

        loop {
            if let Ok(command) = input_out.try_recv() {
                let _ = self.self_in.send(command);
            }

            if let Ok(response) = self.core_out.try_recv() {
                for line in response.split("\n") {
                    println!("=> {}", line.trim());
                }
            }
        }
    }
}

use std::io;
use std::thread;

use types::double_channel::{channel, Endpoint};

pub struct Console {
    link_core: Endpoint<String, String>,
}

impl Console {
    pub fn new(link: Endpoint<String, String>) -> Console {
        Console {
            link_core: link,
        }
    }

    fn non_blocking_stdin(link: Endpoint<String, ()>) {
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
        let (ch_input, ch_me_input) = channel::<String, ()>();

        thread::spawn(move || {
            Console::non_blocking_stdin(ch_input);
        });

        loop {
            if let Ok(command) = ch_me_input.try_recv() {
                let _ = self.link_core.send(command);
            }

            if let Ok(response) = self.link_core.try_recv() {
                for line in response.split("\n") {
                    println!("=> {}", line.trim());
                }
            }
        }
    }
}

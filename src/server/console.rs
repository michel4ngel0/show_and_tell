use std::sync::mpsc::Sender;
use std::io;

pub struct Console {
    core_link: Sender<String>,
}

impl Console {
    pub fn new(link: Sender<String>) -> Console {
        Console {
            core_link: link,
        }
    }

    pub fn run(&self) {
        let mut line = String::new();

        loop {
            if let Ok(_) = io::stdin().read_line(&mut line) {
                let trimmed = String::from(line.trim());
                match self.core_link.send(trimmed) {
                    Ok(_)  => {},
                    Err(_) => println!("(Console) Failed to send a message to core"),
                };
                line.clear();
            }
        }
    }
}

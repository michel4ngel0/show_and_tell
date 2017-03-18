use types::message::Message;

use std::sync::mpsc::Receiver;
use rustc_serialize::json;

pub struct Visualization {
    link: Receiver<Option<Message>>,
    publisher: String,
    config_file: String,
}

impl Visualization {
    pub fn new(link: Receiver<Option<Message>>, publisher: String, config_file: String) -> Visualization {
        Visualization {
            link: link,
            publisher: publisher,
            config_file: config_file,
        }
    }

    pub fn run(&self) {
        loop {
            if let Ok(msg_option) = self.link.try_recv() {
                match msg_option {
                    Some(msg) => {
                        println!("(visualization) received message:\n{}", json::as_pretty_json(&msg));
                    },
                    None      => {
                        println!("(visualization) terminating");
                        break;
                    },
                };
            }
        }
    }
}

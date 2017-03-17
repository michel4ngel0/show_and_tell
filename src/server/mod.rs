pub mod networking;
pub mod console;
pub mod display;

pub struct Server {
    port: u32,
}

impl Server {
    pub fn new(port: u32) -> Server {
        Server{
            port: port,
        }
    }

    pub fn run(&mut self) {
        println!("Listening on port {}", self.port);

        
    }
}

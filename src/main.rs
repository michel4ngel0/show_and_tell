extern crate show_and_tell;

use std::env;
use show_and_tell::server::Server;

const DEFAULT_PORT: u32 = 58196;

fn parse_port(port_string: &String) -> Result<u32, &'static str> {
    let port = match port_string.parse::<u32>() {
        Ok(num) => num,
        Err(_)  => return Err("Invalid port: argument is not a number"),
    };

    if port < 49152 || port > 65535 {
        return Err("Invalid port, use number from range 49152-65535");
    }

    return Ok(port);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 && args.len() != 1 {
        println!("usage: show_and_tell [port]");
        return;
    }

    let mut port: u32 = DEFAULT_PORT;

    if args.len() == 2 {
        match parse_port(&args[1]) {
            Ok(num) => port = num,
            Err(why) => {
                println!("{}", why);
                return;
            }
        }
    }

    let mut server = Server::new(port);
    server.run();
}

extern crate show_and_tell;
extern crate rand;

use std::env;
use show_and_tell::Server;
use rand::Rng;

const MIN_VALID_PORT: u32 = 1024;
const MAX_VALID_PORT: u32 = 49151;

fn parse_port(port_string: &String) -> Result<u32, String> {
    let port = match port_string.parse::<u32>() {
        Ok(num) => num,
        Err(_)  => return Err("Invalid port: argument is not a number".to_string()),
    };

    if port < MIN_VALID_PORT || port > MAX_VALID_PORT {
        let err_msg = format!("Invalid port, use number from range {}-{}", MIN_VALID_PORT, MAX_VALID_PORT);
        return Err(err_msg);
    }

    return Ok(port);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 && args.len() != 1 {
        println!("usage: show_and_tell [port]");
        return;
    }

    let mut port: u32 = rand::thread_rng().gen_range(MIN_VALID_PORT, MAX_VALID_PORT + 1);

    if args.len() == 2 {
        match parse_port(&args[1]) {
            Ok(num) => port = num,
            Err(why) => {
                println!("{}", why);
                return;
            }
        }
    }

    let server = Server::new(port);
    server.run();
}

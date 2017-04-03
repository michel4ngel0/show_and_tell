extern crate show_and_tell;
extern crate rand;

use std::env;
use show_and_tell::Server;
use rand::Rng;
use std::net;
use std::str::FromStr;
use std::error::Error;

const MIN_VALID_PORT: u32 = 1024;
const MAX_VALID_PORT: u32 = 49151;

fn parse_ip(ip_string: &String) -> Result<net::Ipv4Addr, String> {
    match net::Ipv4Addr::from_str(ip_string) {
        Ok(address) => Ok(address),
        Err(error)  => Err(String::from(error.description())),
    }
}

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
    if args.len() != 3 && args.len() != 2 {
        println!("usage: show_and_tell address [port]");
        return;
    }

    let mut port: u32 = rand::thread_rng().gen_range(MIN_VALID_PORT, MAX_VALID_PORT + 1);
    let mut address: net::Ipv4Addr = net::Ipv4Addr::new(127, 0, 0, 1);

    match parse_ip(&args[1]) {
        Ok(addr) => address = addr,
        Err(why) => {
            println!("{}", why);
            return;
        }
    }

    if args.len() == 3 {
        match parse_port(&args[2]) {
            Ok(num) => port = num,
            Err(why) => {
                println!("{}", why);
                return;
            }
        }
    }

    let mut server = Server::new(address, port);
    server.run();
}

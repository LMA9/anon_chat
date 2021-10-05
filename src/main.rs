use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::env::args;
use std::process;

pub mod chat;
pub mod client;

use client::Client;

fn main() {
    let mut port: u16 = 9090;
    let mut addr: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
    let mut name: String = String::from("MyChatClient");
    let args: Vec<String> = args().collect();
    match args.len() {
        1 => {
            println!("Program executed without params, default params was used:\nPORT: {}\nADDRESS: {}\nNAME: {}",
            port, addr, name)
        }
        2 => {
            port = parse_port(&args)
        }
        3 => {
            port = parse_port(&args);
            addr = parse_addr(&args);
        }
        4 => {
            port = parse_port(&args);
            addr = parse_addr(&args);
            name = args[3].clone();
        }
        _ => { panic!("Too much args was given.") }
    }

    let addr = SocketAddr::new(IpAddr::V4(addr), port);
    let mut client = Client::new(name, addr);
    client.run()

}

fn parse_port(args: &Vec<String>) -> u16 {
    match args[1].parse() {
        Ok(entered_port) => entered_port,
        Err(e) => {
            println!("Error to parse port number: {}", e);
            process::exit(1)
        }
    }
}

fn parse_addr(args: &Vec<String>) -> Ipv4Addr {
    match args[2].parse() {
        Ok(addr) => addr,
        Err(e) => {
            println!("Error to parse address: {}", e);
            process::exit(1)
        }
    }
}

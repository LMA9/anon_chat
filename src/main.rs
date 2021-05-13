use std::net::{SocketAddr, Ipv4Addr, IpAddr};

mod users;
mod chat;

use users::User;
use chat::Client;

fn main() {
    // let mut user = User::new(1, "maxon".to_string(), "maxon@mail.com".to_string());
    // if !user.is_protected() {
    //     println!("user {} not Protected", user);
    // }
    // user.set_password(String::from("jopa"));
    // if user.is_protected() {
    //     println!("user {} Protected", user);
    // }
    // println!("Hello, world!");

    let name = "MyClient".to_string();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut client = Client::new(name, addr);
    client.run()

}

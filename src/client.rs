use std::net::{TcpListener, TcpStream, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::io::stdin;
use std::io::prelude::*;
use crate::chat::{Chat};


pub struct Client {
    id: u64,
    name: String,
    socket_addr: SocketAddr,
    listener: Arc<Mutex<TcpListener>>,
    chats: Vec<Chat>,
    connections: Arc<Mutex<Vec<TcpStream>>>
}

impl Client {
    pub fn new(name: String, socket_addr: SocketAddr) -> Client {
        let generated_id = 0;
        let listener = TcpListener::bind(socket_addr).unwrap();
        // listener.set_nonblocking(true).expect("Error with set nonblocking mode for listener.");

        Client {
            id: generated_id,
            name,
            socket_addr,
            listener: Arc::new(Mutex::new(listener)),
            chats: Vec::new(),
            connections: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn run(&mut self) {
        'running: loop {
            let selected_option = Client::main_menu();
            match selected_option {
                0 => {
                    println!("Quiting...");
                    break 'running
                },
                1 => self.listen_connections(),
                2 => self.accept_connection_request(),
                3 => self.send_chat_request(),
                4 => {
                    match self.select_chat() {
                        Some(chat) => chat.start(),
                        None => {}
                    }
                },
                _ => {
                    println!("Unknown option. Quiting...");
                    break 'running
                }
            }
        }
        // self.stop_listener();
    }

    // fn stop_listener(&mut self) {
    //     match self.listener_handler {
    //         Some(handler) => {
    //             self.listener_handler = None;
    //             handler.join().unwrap();
    //         },
    //         None => println!("Listener hasn't be started.")
    //     }
    // }

    pub fn main_menu() -> usize {
        let menu_options = [
            (1, "Listen conections."),
            (2, "Accept chat request."),
            (3, "Send chat request."),
            (4, "Select chat."),
            (0, "Exit"),
        ];
        loop {
            println!("\u{001b}[2JSelect menu option:\n{}", menu_options.iter().map(|(i, s)| format!("{}. {}", i, s)).collect::<Vec<String>>().join("\n"));
            let mut selected_option = String::new();
            stdin().read_line(&mut selected_option).unwrap();
            selected_option = selected_option.trim().to_string();
            match selected_option.parse::<usize>() {
                Ok(n) => {
                    for (index, _) in menu_options.iter() {
                        if n == *index {
                            return n
                        }
                    }
                    println!("Error: Incorrect option!")
                },
                Err(e) => {
                    println!("Error: Incorrect option!\n{}", e)
                }
            }
        }
    }

    fn send_self_creds(&self, stream: &mut TcpStream) {
        let mut data = Vec::from(self.id.to_be_bytes());
        data.append(&mut self.name.as_bytes().to_vec());
        stream.write(&mut data).unwrap();
    }

    fn send_chat_request(&mut self) {
        println!("Enter address to connection(e.g: 10.10.14.132:777):\n");
        let mut connection_addr = String::new();
        stdin().read_line(&mut connection_addr).unwrap();
        connection_addr = connection_addr.trim().to_string();
        match connection_addr.parse::<SocketAddrV4>() {
            Ok(addr) => {
                if let Ok(mut stream) = TcpStream::connect(addr) {
                    self.send_self_creds(&mut stream);
                    println!("New connection created! Please wait for opponent accept request...");

                    if let Some(new_chat) = Chat::from_tcp_stream(stream) {
                        {
                            let mut stream = new_chat.stream.lock().unwrap();
                            self.send_self_creds(&mut stream);
                        }
                        println!("Chat with {} name was created!", new_chat.name);
                        self.chats.push(new_chat)
                    } else {
                        println!("Unable to accept opponent creds. Aborting...")
                    }
                } else {
                    println!("Could not connect to this address. Aborting...")
                }
            },
            Err(e) => println!("Wrong connection address: {}", e)
        }
    }

    fn accept_connection_request(&mut self) {
        let mut connections = self.connections.lock().unwrap();
        if connections.is_empty() {
            println!("You have no requests.")
        } else {
            println!("Accept request from:");
            for (i, stream) in connections.iter().enumerate() {
                println!("{}. {}", i, stream.peer_addr().unwrap() )
            }
            let mut selected_request = String::new();
            stdin().read_line(&mut selected_request).unwrap();
            selected_request = selected_request.trim().to_string();
            match selected_request.parse::<usize>() {
                Ok(n) => {
                    let stream = connections.remove(n);
                    if let Some(new_chat) = Chat::from_tcp_stream(stream) {
                        {
                            let mut stream = new_chat.stream.lock().unwrap();
                            self.send_self_creds(&mut stream);
                        }
                        println!("Chat with {} name was created!", new_chat.name);
                        self.chats.push(new_chat)
                    } else {
                        println!("Unable to accept request. Aborting...")
                    };
                },
                Err(e) => {
                    println!("Error: Incorrect option!\n{}", e)
                }
            }
        }
    }

    // fn toggle_connection_listening(&mut self) {
    //     match &self.listener_handler {
    //         Some(handler) => self.listener_handler = None,
    //         None => {
    //             let listener = self.listener.clone();
    //             let requests = self.requests.clone();
    //             self.listener_handler = Some(thread::spawn(move || {
    //                 Client::listen_connections(listener, requests)
    //             }));
    //         }
    //     }
    //     return
    // }

    fn listen_connections(&mut self) {
        println!("New connections listener was started...");
        'listening: loop {
            let listener = self.listener.lock().unwrap();
    
            // Проверка новых подключений
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!("New connection with {}", addr);
                    let mut requests = self.connections.lock().unwrap();
                    requests.push(stream);
                    break 'listening
                }
                Err(e) => println!("Connection failed: {}", e),
            }
        }
    }

    fn select_chat(&mut self) -> Option<&mut Chat> {
        if self.chats.is_empty() {
            println!("You have no chats.");
            return None
        } else {
            println!("Select chat:");
                for (i, chat) in self.chats.iter().enumerate() {
                    println!("{}. {}", i, chat.name)
                }
                let mut selected_chat = String::new();
                stdin().read_line(&mut selected_chat).unwrap();
                selected_chat = selected_chat.trim().to_string();
                match selected_chat.parse::<usize>() {
                    Ok(n) => {
                        for (index, chat) in self.chats.iter_mut().enumerate() {
                            if n == index {
                                return Some(chat)
                            }
                        }
                        println!("Error: Incorrect option!");
                        return None
                    },
                    Err(e) => {
                        println!("Error: Can't parse option!\n{}", e);
                        return None
                    }
                }
        }
    }
}
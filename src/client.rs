use std::net::{TcpListener, TcpStream, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::io::stdin;
use std::io::prelude::*;
use crate::chat::{Chat};

struct Listener {
    tcp_listener: Arc<Mutex<TcpListener>>,
    is_listen: Arc<Mutex<bool>>,
    connections: Arc<Mutex<Vec<TcpStream>>>
}

impl Listener {
    fn init(socket_addr: SocketAddr, connections: Arc<Mutex<Vec<TcpStream>>>) -> Self {
        let tcp_listener = Arc::new(Mutex::new(TcpListener::bind(socket_addr).unwrap()));
        let listener = Self {
            tcp_listener,
            connections,
            is_listen: Arc::new(Mutex::new(false)),
        };
        listener.start_listener();
        listener
    }

    fn set_listening(&mut self, listen: bool) {
        *self.is_listen.lock().unwrap() = listen;
        if listen {
            println!("Listening was started...")
        } else {
            println!("Listening was stoped...")
        }
        std::thread::sleep(std::time::Duration::from_millis(700))
    }
    
    fn is_listen(&self) -> bool {
        *self.is_listen.lock().unwrap()
    }

    fn start_listener(&self) {
        let listener = self.tcp_listener.clone();
        let connections = self.connections.clone();
        let is_listen = self.is_listen.clone();

        std::thread::spawn(move || {
            loop {
                let listener = listener.lock().unwrap();
            
                // Проверка новых подключений
                match listener.accept() {
                    Ok((stream, addr)) => {
                        if *is_listen.lock().unwrap() {
                            println!("New connection with {}", addr);
                            let mut requests = connections.lock().unwrap();
                            requests.push(stream);
                        } else {
                            drop(stream);
                        }
                    }
                    Err(e) => println!("Connection failed: {}", e),
                }
            }
        });
        std::thread::sleep(std::time::Duration::from_secs(1))
    }
}


pub struct Client {
    id: u64,
    name: String,
    listener: Listener,
    chats: Vec<Chat>,
    connections: Arc<Mutex<Vec<TcpStream>>>
}

impl Client {
    pub fn new(name: String, socket_addr: SocketAddr) -> Client {
        let generated_id = 0;
        let connections = Arc::new(Mutex::new(Vec::new()));

        Client {
            id: generated_id,
            name,
            listener: Listener::init(socket_addr, connections.clone()),
            chats: Vec::new(),
            connections: connections
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
                1 => self.toggle_connections_listening(),
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
    }

    pub fn toggle_connections_listening(&mut self) {
        self.listener.set_listening(!self.listener.is_listen())
    }

    pub fn main_menu() -> usize {
        let menu_options = [
            (1, "Toggle conections listening."),
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
                        // {
                        //     let mut stream = new_chat.stream.lock().unwrap();
                        //     self.send_self_creds(&mut stream);
                        // }
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
        std::thread::sleep(std::time::Duration::from_secs(1))
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
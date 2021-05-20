use std::net::{TcpListener, TcpStream, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::SystemTime;
use std::thread;
use std::sync::{Arc, Mutex};
use std::io::stdin;
use std::io::prelude::*;

struct Connection {
    addr: SocketAddr,
    stream: TcpStream
}

impl Connection {
    fn new(addr: SocketAddr, stream: TcpStream) -> Connection {
        Connection { addr, stream }
    }
}

struct Chat<'a> {
    id: u64,
    name: String,
    connection: Connection,
    messages: Vec<Message<'a>>
}

impl<'a> Chat<'a> {
    fn new(id: u64, name: String, connection: Connection) -> Chat<'a> {
        Chat {
            id,
            name,
            connection,
            messages: Vec::new()
        }
    }

    fn create_message(&'a self, sender_id: u64, text: String) -> Message<'a> {
        Message::new(self, sender_id, text)
    }

    fn add_message(&mut self, message: Message<'a>) {
        self.messages.push(message)
    }
}

struct Message<'a> {
    chat: &'a Chat<'a>,
    sender_id: u64,
    text: String,
    date: SystemTime,
}

impl<'a> Message<'a> {
    fn new(chat: &'a Chat<'a>, sender_id: u64, text: String) -> Message<'a> {
        Message {
            chat,
            sender_id,
            text,
            date: SystemTime::now(),
        }
    }
}

pub struct Client<'a> {
    id: u64,
    name: String,
    socket_addr: SocketAddr,
    listener: TcpListener,
    chats: Vec<Chat<'a>>,
    requests: Vec<Connection>,
}

impl<'a> Client<'a> {
    pub fn new(name: String, socket_addr: SocketAddr) -> Client<'a> {
        let generated_id = 0;
        let listener = TcpListener::bind(socket_addr).unwrap();
        Client {
            id: generated_id,
            name,
            socket_addr,
            listener,
            chats: Vec::new(),
            requests: Vec::new()
        }
    }

    pub fn run(& mut self) {
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
                        Some(chat) => {
                            println!("Selected {} chat", chat.name)
                        },
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

    pub fn main_menu() -> usize {
        let menu_options = [
            (0, "Exit"),
            (1, "Listen conections."),
            (2, "Accept chat request."),
            (3, "Send chat request."),
            (4, "Select chat."),
        ];
        loop {
            println!("Select menu option:\n{}", menu_options.iter().map(|(i, s)| format!("{}. {}", i, s)).collect::<Vec<String>>().join("\n"));
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
        println!("Please, enter address to connection(e.g: 10.10.14.132:777):\n");
        let mut connection_addr = String::new();
        stdin().read_line(&mut connection_addr).unwrap();
        connection_addr = connection_addr.trim().to_string();
        match connection_addr.parse::<SocketAddrV4>() {
            Ok(addr) => {
                if let Ok(mut stream) = TcpStream::connect(addr) {
                    self.send_self_creds(&mut stream);
                    let new_connection = Connection::new(SocketAddr::V4(addr), stream);
                    println!("New connection created! Please wait for opponent accept request...");
                    
                    if let Some(mut new_chat) = Client::parse_request(new_connection) {
                        self.send_self_creds(&mut new_chat.connection.stream);
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
        if self.requests.is_empty() {
            println!("You have no requests.")
        } else {
            println!("Accept request from:");
            for (i, request) in self.requests.iter().enumerate() {
                println!("{}. {}", i, request.addr )
            }
            let mut selected_request = String::new();
            stdin().read_line(&mut selected_request).unwrap();
            selected_request = selected_request.trim().to_string();
            match selected_request.parse::<usize>() {
                Ok(n) => {
                    let request = self.requests.remove(n);
                    if let Some(mut chat) = Client::parse_request(request) {
                        self.send_self_creds(&mut chat.connection.stream);
                        println!("Chat with {} name was created!", chat.name);
                        self.chats.push(chat)
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

    fn parse_request(mut connection: Connection) -> Option<Chat<'a>> {
        let mut id_buffer: [u8; 8] = [0; 8];
        let target_id: u64;
        let s = &mut connection.stream;
        let mut handler = s.take(8);
        handler.read_exact(&mut id_buffer).expect("Failed to parse chat ID");
        target_id = u64::from_be_bytes(id_buffer);
        println!("ID was parsed: {}", target_id);
        
        let mut name_buffer: [u8; 32] = [0; 32];
        let target_name: String;
        let s = &mut connection.stream;
        let mut handler = s.take(32);
        match handler.read(&mut name_buffer) {
            Ok(n) => {
                match String::from_utf8(name_buffer[..n].to_vec()) {
                    Ok(name) => target_name = name,
                    Err(e) => {
                        println!("Error with reading Name from {}: {}", connection.addr, e);
                        return None
                    }
                }
            },
            Err(e) => {
                println!("Error with reading Name from {}: {}", connection.addr, e);
                return None
            }
        }

        Some(Chat::new(target_id, target_name, connection))
    }

    fn listen_connections(&mut self) {
        println!("Listening connections...");
        'listening: loop {

            // Проверка новых подключений
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    println!("New connection with {}", addr);
                    self.requests.push(Connection::new(addr, stream));
                    break 'listening
                }
                Err(e) => {println!("Connection failed: {}", e)},
            }


        }
    }

    fn select_chat(&mut self) -> Option<&Chat> {
        if self.chats.is_empty() {
            println!("You have no chats.");
            return None
        } else {
            loop {
                println!("Select chat:");
                for (i, chat) in self.chats.iter().enumerate() {
                    println!("{}. {}", i, chat.name)
                }
                let mut selected_chat = String::new();
                stdin().read_line(&mut selected_chat).unwrap();
                selected_chat = selected_chat.trim().to_string();
                match selected_chat.parse::<usize>() {
                    Ok(n) => {
                        for (index, chat) in self.chats.iter().enumerate() {
                            if n == index {
                                return Some(chat)
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
    }
}

extern crate chrono;

use chrono::{DateTime, Local};
use std::net::{TcpListener, TcpStream, SocketAddr, SocketAddrV4};
use std::sync::{Arc, Mutex};
use std::io::stdin;
use std::io::prelude::*;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver, RecvError};

struct Connection {
    addr: SocketAddr,
    stream: TcpStream
}

impl Connection {
    fn new(addr: SocketAddr, stream: TcpStream) -> Connection {
        Connection { addr, stream }
    }

    fn into_chat<'a>(mut self) -> Option<Chat> {
        let mut id_buffer: [u8; 8] = [0; 8];
        let target_id: u64;
        let s = &mut self.stream;
        let mut handler = s.take(8);
        handler.read_exact(&mut id_buffer).expect("Failed to parse chat ID");
        target_id = u64::from_be_bytes(id_buffer);
        println!("ID was parsed: {}", target_id);
        
        let mut name_buffer: [u8; 32] = [0; 32];
        let target_name: String;
        let s = &mut self.stream;
        let mut handler = s.take(32);
        match handler.read(&mut name_buffer) {
            Ok(n) => {
                match String::from_utf8(name_buffer[..n].to_vec()) {
                    Ok(name) => target_name = name,
                    Err(e) => {
                        println!("Error with reading Name from {}: {}", self.addr, e);
                        return None
                    }
                }
            },
            Err(e) => {
                println!("Error with reading Name from {}: {}", self.addr, e);
                return None
            }
        }
        self.stream.set_nonblocking(true).unwrap();
        Some(Chat::new(target_id, target_name, self))
    }
}

struct Chat {
    id: u64,
    name: String,
    connection: Connection,
    messages: Vec<Message>
}

impl Chat {
    fn new(id: u64, name: String, connection: Connection) -> Chat {
        Chat {
            id,
            name,
            connection,
            messages: Vec::new()
        }
    }

    fn add_message(&mut self, message: Message) {
        let sender = if self.id == message.chat_id {
            format!("{}({})", self.name, self.id)
        } else {
            String::from("You")
        };
        println!("{}: {} ({})", sender, message.text, message.date.format("%H:%M:%S"));
        self.messages.push(message)
    }

    fn check_new_message(chat_id: u64, stream: &TcpStream) -> Option<Message> {
        let mut message_buffer: [u8; 1024] = [0; 1024];
        let mut handler = stream.take(1024);
        // Блокирует основной поток
        println!("Reading");
        match handler.read(&mut message_buffer) {
            Ok(n) => {
                if n == 0 {
                    println!("Zero");
                    return None
                }
                match String::from_utf8(message_buffer[..n].to_vec()) {
                    Ok(message) => {
                        let message = message.trim_end().to_string();
                        println!("Readed");
                        Some(Message::new(chat_id, message))
                    },
                    Err(e) => {
                        println!("Error with reading message from {}: {}", chat_id, e);
                        return None
                    }
                }
            },
            Err(e) => {
                println!("Error with reading message from {}: {}", chat_id, e);
                return None
            }
        }
    }

    fn send_message(&mut self, message: String) {
        let mut message_data = message.as_bytes().to_vec();
        self.connection.stream.write(&mut message_data).unwrap();
    }
    
    fn start(&mut self) {
        println!("Chat with {}({}) was started", self.name, self.id);
        // Попробовать через Arc и Mutex
        let stream = &self.connection.stream;
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            loop {
                match Chat::check_new_message(self.id, stream) {
                    Some(message) => {
                        tx.send(message).unwrap();
                    }
                }
            }
        });
        'chat: loop {

            let mut my_message = String::new();
            // print!("Введите сообщение: ");
            stdin().read_line(&mut my_message).unwrap();
            my_message = my_message.trim_end().to_string();
            if !my_message.is_empty() {
                if my_message == "/exit" {
                    break 'chat
                } else {
                    println!("You: {} ({})", my_message, Local::now().format("%H:%M:%S"));
                    self.send_message(my_message);
                }
            }
        }
    }
}

struct Message {
    chat_id: u64,
    text: String,
    date: DateTime<Local>,
}

impl Message {
    fn new(chat_id: u64, text: String) -> Message {
        Message {
            chat_id,
            text,
            date: Local::now(),
        }
    }
}

pub struct Client {
    id: u64,
    name: String,
    socket_addr: SocketAddr,
    listener: Arc<Mutex<TcpListener>>,
    chats: Vec<Chat>,
    requests: Arc<Mutex<Vec<Connection>>>
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
            requests: Arc::new(Mutex::new(Vec::new()))
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

                    if let Some(mut new_chat) = new_connection.into_chat() {
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
        let mut requests = self.requests.lock().unwrap();
        if requests.is_empty() {
            println!("You have no requests.")
        } else {
            println!("Accept request from:");
            for (i, request) in requests.iter().enumerate() {
                println!("{}. {}", i, request.addr )
            }
            let mut selected_request = String::new();
            stdin().read_line(&mut selected_request).unwrap();
            selected_request = selected_request.trim().to_string();
            match selected_request.parse::<usize>() {
                Ok(n) => {
                    let request = requests.remove(n);
                    if let Some(mut chat) = request.into_chat() {
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
                    let mut requests = self.requests.lock().unwrap();
                    requests.push(Connection::new(addr, stream));
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

extern crate chrono;

use chrono::{DateTime, Local};
use std::net::{TcpStream, SocketAddr};
use std::io::stdin;
use std::io::prelude::*;
use std::sync::mpsc::{channel, Receiver, RecvError};

pub struct Chat {
    id: u64,
    pub name: String,
    pub connection: Connection,
    messages: Vec<Message>
}

impl Chat {
    pub fn new(id: u64, name: String, connection: Connection) -> Chat {
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
    
    pub fn start(&mut self) {
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

pub struct Connection {
    pub addr: SocketAddr,
    pub stream: TcpStream
}

impl Connection {
    pub fn new(addr: SocketAddr, stream: TcpStream) -> Connection {
        Connection { addr, stream }
    }

    pub fn into_chat<'a>(mut self) -> Option<Chat> {
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

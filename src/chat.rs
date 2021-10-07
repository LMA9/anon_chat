extern crate chrono;

use chrono::{DateTime, Local};
use std::net::{TcpStream};
use std::time::Duration;
use std::thread;
use std::io::stdin;
use std::io::prelude::*;
use std::sync::mpsc::{channel};
use std::sync::{Arc, Mutex};
use std::fmt;

pub enum ChatCloseType {
    Minimize,
    Close
}

pub struct Chat {
    id: u64,
    pub name: String,
    pub stream: Arc<Mutex<TcpStream>>,
    messages: Arc<Mutex<Vec<Message>>>
}

impl Chat {
    pub fn new(id: u64, name: String, stream: TcpStream) -> Self {
        Self {
            id,
            name,
            stream: Arc::new(Mutex::new(stream)),
            messages: Arc::new(Mutex::new(Vec::new()))
        }
    }

    pub fn from_tcp_stream(mut stream: TcpStream) -> Option<Chat> {
        let mut id_buffer: [u8; 8] = [0; 8];
        let target_id: u64;
        let s = &mut stream;
        let mut handler = s.take(8);
        match handler.read_exact(&mut id_buffer) {
            Ok(()) => {},
            Err(e) => {
                println!("Failed to parse chat ID: {}", e);
                return None
            }
        }
        target_id = u64::from_be_bytes(id_buffer);
        println!("ID was parsed: {}", target_id);
        
        let mut name_buffer: [u8; 32] = [0; 32];
        let target_name: String;
        let s = &mut stream;
        let mut handler = s.take(32);
        match handler.read(&mut name_buffer) {
            Ok(n) => {
                target_name = match String::from_utf8(name_buffer[..n].to_vec()) {
                    Ok(name) => name,
                    Err(e) => {
                        println!("Error with reading Name from {:?}: {}", stream.peer_addr(), e);
                        return None
                    }
                }
            },
            Err(e) => {
                println!("Error with reading Name from {:?}: {}", stream.peer_addr(), e);
                return None
            }
        }
        stream.set_nonblocking(true).unwrap();
        Some(Chat::new(target_id, target_name, stream))
    }

    fn check_new_message(chat_id: u64, stream: &Arc<Mutex<TcpStream>>) -> Option<Message> {
        let mut message_buffer: [u8; 1024] = [0; 1024];
        let s = &mut stream.lock().unwrap();
        match s.read(&mut message_buffer) {
            Ok(n) => {
                if n == 0 {
                    return None
                }
                match String::from_utf8(message_buffer[..n].to_vec()) {
                    Ok(message) => {
                        let message = message.trim_end().to_string();
                        Some(Message::new(chat_id, message))
                    },
                    Err(e) => {
                        println!("Error with reading message bytes: {}", e);
                        return None
                    }
                }
            },
            Err(_e) => {
                return None
            }
        }
    }

    fn send_message(&mut self, message: String) {
        let mut message_data = message.as_bytes().to_vec();
        let mut stream = self.stream.lock().unwrap();
        stream.write(&mut message_data).unwrap();
    }
    
    pub fn start(&mut self) {
        println!("Chat with {}({}) was started", self.name, self.id);
        let stream = self.stream.clone();
        let (exit_sender, exit_receiver) = channel();
        let chat_id = self.id;
        let chat_name = self.name.clone();
        let messages = self.messages.clone();
        let checker = thread::spawn(move || {
            loop {
                match Chat::check_new_message(chat_id, &stream) {
                    Some(message) => {
                        println!("{}({}): {}", chat_name, chat_id, message);
                        let mut msgs = messages.lock().unwrap();
                        msgs.push(message);
                    },
                    None => { thread::sleep(Duration::from_secs(1)) }
                }
                match exit_receiver.try_recv() {
                    Ok(_) => {
                        break
                    },
                    Err(_) => {}
                }
            }
            drop(exit_receiver)
        });
        'chat: loop {
            let mut my_message = String::new();
            stdin().read_line(&mut my_message).unwrap();
            if my_message.ends_with("\n") {
                my_message = my_message.trim_end().to_string();
                if !my_message.is_empty() {
                    if my_message == "/exit" {
                        exit_sender.send(true).unwrap();
                        break 'chat
                    } else {
                        println!("You: {} ({})", my_message, Local::now().format("%H:%M:%S"));
                        self.send_message(my_message);
                    }
                }
            }
        }
        checker.join().unwrap();
    }
}

#[derive(Clone, Debug)]
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

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.text, self.date.format("%H:%M:%S"))
    }
}

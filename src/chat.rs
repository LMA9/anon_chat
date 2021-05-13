use std::net::{TcpListener, TcpStream, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::SystemTime;
use std::thread;
use std::io::stdin;

struct Connection {
    addr: SocketAddr,
    stream: TcpStream
}

struct Chat<'a> {
    id: u64,
    connection: Connection,
    messages: Vec<Message<'a>>
}

impl<'a> Chat<'a> {
    fn new(id: u64, connection: Connection) -> Chat<'a> {
        Chat {
            id,
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
    requests: Vec<Request>,
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

    fn listen_connections(&mut self) {
        println!("Listening connections...");
        'listening: loop {

            // Проверка новых подключений
            match self.listener.accept() {
                Ok((socket, addr)) => {
                    println!("New connection with {}", addr);
                    self.requests.push(Request { socket, addr });
                    break 'listening
                }
                Err(e) => {println!("Connection failed: {}", e)},
            }


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
                _ => {
                    println!("Unknown option. Quiting...");
                    break 'running
                }
            }
        }
    }

    pub fn main_menu() -> u8 {
        let menu_options = [
            (0, "Exit"),
            (1, "Listen conection."),
        ];
        loop {
            println!("Select menu option:\n{}", menu_options.iter().map(|(i, s)| format!("{}. {}", i, s)).collect::<Vec<String>>().join("\n"));
            let mut selected_option = String::new();
            stdin().read_line(&mut selected_option).unwrap();
            selected_option = selected_option.trim().to_string();
            match selected_option.parse::<u8>() {
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
}

struct Request {
    socket: TcpStream,
    addr: SocketAddr,
}

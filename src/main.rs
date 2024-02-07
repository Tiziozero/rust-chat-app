use std::net::{Shutdown, TcpListener, TcpStream};
use std::{io, thread};
use std::sync::{mpsc, Arc, Mutex};
use uuid::Uuid;
use std::io::{Read, Write};

use std::collections::HashMap;

#[derive(Debug)]
struct Message {
    name: String,
    content: String,
    message_id: String,
}

#[derive(Debug)]
struct Client {
    name: String,
    token: String,
    stream: TcpStream,
}

impl Client {
    fn clone(&self) -> Result<Self, String> {
        Ok(
            Client {
                name: self.name.clone(),
                token: self.token.clone(),
                stream: self.stream.try_clone().unwrap()
            }
        )
    }
}


fn handle_client(tx: mpsc::Sender<Message>,mut client: Client) -> std::io::Result<()> {
    loop {
        let mut buf = [0u8; 1024];
        let amt = match client.stream.read(&mut buf) {
            Ok(0) => {
                eprintln!("Received 0 from {}. Disconnecting.", client.stream.peer_addr()?);
                client.stream.shutdown(Shutdown::Both)?;
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Receivedd 0 from client"));
            },
            Ok(n) => { n },
            Err(_) => {
                eprintln!("Error for {}. Disconnecting.",
                    client.stream.peer_addr()?);
                client.stream.shutdown(Shutdown::Both)?;
                return Err(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Unexpected error"
                        )
                    );
            },
        };
        if let Ok(txt) = std::str::from_utf8(&buf[..amt]) {
            let msg = Message{
                name: client.name.clone(),
                content: txt.to_string(),
                message_id: Uuid::new_v4().to_string()
            };
            match tx.send(msg) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Received 0 from {}. Disconnecting.", client.stream.peer_addr()?);
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Error sending to tx"));
                },
            }
        } else {
            eprintln!("Error in decoding message");
        };
    }
}


fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    let (tx, rx) = mpsc::channel::<Message>();
    let mut clients = Arc::new(Mutex::new(HashMap::<String, Client>::new()));
    let clients_arc = Arc::clone(&clients);

    let _ = thread::spawn( move || {
        loop {
            println!("Receiving in rx");
            let msg = rx.recv().unwrap();
            println!("{:?}", msg);
            let m = format!("[ {} ]::< {} >", msg.name, msg.content);
            let message = m.into_bytes();

            let mut clients = clients_arc.lock().unwrap();
            let mut remove_v: Vec<String> = Vec::new();
            /*for (_ , mut c) in v.iter_mut() {
                    match c.stream.write_all(&message) {
                            Ok(_) => {
                                    println!("wrote to client {}", c.stream.local_addr().unwrap());
                            },
                            Err(e) => {
                                    if e.kind() == io::ErrorKind::BrokenPipe {
                                            eprintln!("Broken PipeLine Error from {}. Disconnecting.", c.stream.local_addr().unwrap());
                                    }
                                    remove_v.push(c.token.clone());
                            },
                    }
            }
            for c in v.iter() {
                    let _ = v.remove(&c);
                    println!("Removed client with token {}", &c);
                                    
            }*/
            for (token, client) in clients.iter_mut() {
                match client.stream.write_all(&message) {
                    Ok(_) => println!("Wrote to client {}", client.stream.local_addr().unwrap()),

                    Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                        eprintln!("Broken Pipe Error from {}. Disconnecting.", client.stream.local_addr().unwrap());
                        remove_v.push(token.clone());
                    },

                    Err(e) => eprintln!("Error writing to client {}: {}", client.stream.local_addr().unwrap(), e),
                }
            }

            for token in remove_v {
                if clients.remove(&token).is_some() {
                    println!("Removed client with token {}", token);

                }
            }
        }

    } );
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let tx = tx.clone();
                let token = Uuid::new_v4().to_string();
                let c: Client = Client {
                    name: String::from("Anon"),
                    token,
                    stream: s,
                };

                clients.lock().unwrap().insert(c.token.clone(), c.clone().unwrap());

                let _ = thread::spawn( move || {
                    let _ = handle_client(tx, c.clone().unwrap());
                });
            },
            Err(_) => {},
        }
    }

    Ok(())
}


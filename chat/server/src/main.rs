use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

const LOCAL: &str = "127.0.0.1:5000";
const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("failed to start non-blocking");

    // Keep track of connected clients and their nicknames
    let mut clients: HashMap<usize, (String, std::net::TcpStream)> = HashMap::new();
    let (tx, rx) = mpsc::channel::<(usize, String)>();
    let mut client_id = 0;

    println!("Server running on {}", LOCAL);

    loop {
        // Accept new clients
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client connected from {}", addr);
            let tx = tx.clone();
            let id = client_id;
            client_id += 1;

            // Clone the socket for storing in the clients map
            let socket_clone = socket.try_clone().expect("Failed to clone socket");
            clients.insert(id, ("Unknown".to_string(), socket_clone));

            // Ask the client for a nickname when they connect
            thread::spawn(move || {
                let mut buff = vec![0; MSG_SIZE];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let nickname = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let nickname = String::from_utf8(nickname).expect("Invalid UTF-8");

                        println!("Client {} connected as '{}'", addr, nickname);

                        // Send a welcome message to the new client
                        tx.send((id, format!("{} has joined the chat!", nickname)))
                            .expect("Failed to send join message");

                        // Main loop for receiving messages from the client
                        loop {
                            let mut buff = vec![0; MSG_SIZE];
                            match socket.read_exact(&mut buff) {
                                Ok(_) => {
                                    let msg = buff
                                        .into_iter()
                                        .take_while(|&x| x != 0)
                                        .collect::<Vec<_>>();
                                    let msg = String::from_utf8(msg).expect("Invalid UTF-8");

                                    // Send the message along with the client's nickname
                                    tx.send((id, format!("{}: {}", nickname, msg)))
                                        .expect("Failed to send message");
                                }
                                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                                Err(_) => {
                                    println!("Client {} disconnected", nickname);
                                    tx.send((id, format!("{} has left the chat", nickname)))
                                        .expect("Failed to send disconnect message");
                                    break;
                                }
                            }
                            sleep();
                        }
                    }
                    Err(_) => println!("Failed to get nickname from {}", addr),
                }
            });
        }

        // Broadcast messages to all connected clients
        if let Ok((sender_id, msg)) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|(id, (nickname, mut client))| {
                    if id == sender_id {
                        return Some((id, (nickname, client))); // Do not send the message back to the sender
                    }
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    client
                        .write_all(&buff)
                        .map(|_| (id, (nickname, client)))
                        .ok()
                })
                .collect::<HashMap<_, _>>();
        }

        sleep();
    }
}

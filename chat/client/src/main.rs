use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:5000";
const MSG_SIZE: usize = 32;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("failed to initiate non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    // Ask for the user's nickname
    println!("Enter your nickname:");
    let mut nickname = String::new();
    io::stdin()
        .read_line(&mut nickname)
        .expect("Failed to read nickname");
    let nickname = nickname.trim().to_string();

    // Send the nickname to the server
    let mut buff = nickname.clone().into_bytes();
    buff.resize(MSG_SIZE, 0);
    client.write_all(&buff).expect("Failed to send nickname");

    // Spawn a thread to handle incoming messages from the server
    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid UTF-8 message");
                println!("{}", msg);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection to the server was severed.");
                break;
            }
        }

        // Handle user-sent messages
        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Failed to write to socket");
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(100));
    });

    // Main loop for user input
    println!("You can start chatting (type ':quit' to exit):");
    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Failed to read from stdin");
        let msg = buff.trim().to_string();

        // Exit if the user types ':quit'
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }

    println!("Goodbye!");
}

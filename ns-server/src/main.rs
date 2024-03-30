use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use ns_core::models::packets::Packet;

fn main() {
    // bind the server to the local address
    let server = TcpListener::bind("127.0.0.1:6666").unwrap();

    // listen for incoming connections
    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                _ => {
                    println!("Error: {}", e);
                    break;
                }
            },
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    loop {
        // u32
        let mut length_header = [0u8; 4];
        stream.read_exact(&mut length_header).unwrap();
        let length = u32::from_le_bytes(length_header);

        // Packet
        let mut buffer = vec![0u8; length as usize];
        stream.read_exact(&mut buffer).unwrap();
        stream.flush().unwrap();

        let packet = Packet::try_from_bytes(&buffer);

        if let Ok(packet) = packet {
            match packet {
                Packet::Disconnect(nickname) => {
                    println!("{} disconnected", nickname);
                    break;
                }
                _ => {
                    println!("{packet}");
                }
            }
        } else if let Err(e) = packet {
            eprintln!("Error: {}", e);
        }
    }
}

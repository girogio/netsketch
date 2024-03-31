mod models;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use models::state::ServerState;
use ns_core::models::{canvas::CanvasAction, packets::Packet};

fn main() {
    // bind the server to the local address
    let server = TcpListener::bind("127.0.0.1:6666").unwrap();
    let server_state = Arc::new(Mutex::new(ServerState::new()));

    // listen for incoming connections
    for stream in server.incoming() {
        let server_state = server_state.clone();
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_client(stream, server_state);
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

fn handle_client(mut stream: TcpStream, server_state: Arc<Mutex<ServerState>>) {
    loop {
        // u32
        let mut length_header = [0u8; 4];
        stream.read_exact(&mut length_header).unwrap();
        let length = u32::from_le_bytes(length_header);

        // Packet
        let mut buffer = vec![0u8; length as usize];
        stream.read_exact(&mut buffer).unwrap();
        stream.flush().unwrap();

        let packet = if let Ok(packet) = Packet::try_from_bytes(&buffer) {
            packet
        } else {
            eprintln!("Malformed packet: {:?}", buffer);
            continue;
        };

        // Reserve the server state for this thread
        let mut server_state = server_state.lock().unwrap();

        match packet {
            Packet::Connect(nickname) => {
                server_state.connect_user(&stream, nickname);
                let update_packet = Packet::LoadCanvas(server_state.canvas.actions.clone());
                let packet_bytes = update_packet.to_bytes();
                stream.write_all(&packet_bytes).unwrap();
            }

            Packet::Disconnect => {
                server_state.disconnect_user(&stream);
                break;
            }

            Packet::Draw(action) => {
                // Draw on the server's canvas
                let current_user = server_state.get_username(&stream);

                let new_entry = server_state.canvas.add_action(current_user, &action);

                // Send the update to all connected clients
                let update_packet = Packet::UpdatePeers(new_entry);
                for connection in server_state.connections.iter_mut() {
                    let packet_bytes = update_packet.to_bytes();
                    connection.write_all(&packet_bytes).unwrap();
                }
            }
            _ => {}
        }
    }
}

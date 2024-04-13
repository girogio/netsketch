mod models;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::Parser;
use models::{state::ServerState, user_data::Action};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use ns_core::errors::Result;
use ns_core::models::packets::TcpPacket;

use crate::models::user_data;

#[derive(Parser)]
struct Args {
    /// The address of the netsketch server
    #[clap(short, long)]
    address: String,
    /// The port of the netsketch server
    #[clap(short, long)]
    port: u16,
}

fn main() {
    let args = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // bind the server to the local address
    let server = match TcpListener::bind(format!("{}:{}", args.address, args.port)) {
        Ok(server) => server,
        Err(_) => {
            error!(
                "Failed to bind the server to {}:{}",
                args.address, args.port
            );
            std::process::exit(1);
        }
    };

    let server_state = Arc::new(Mutex::new(ServerState::new()));

    // listen for incoming sessions
    for stream in server.incoming() {
        let server_state = server_state.clone();
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    let stream = Arc::new(stream);
                    loop {
                        handle_client(stream.clone(), server_state.clone());
                    }
                });
            }
            Err(e) => {
                error!("{e}", e = e.kind());
                continue;
            }
        }
    }
}

fn handle_client(stream: Arc<TcpStream>, server_state: Arc<Mutex<ServerState>>) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(600)))?;

    // Payload length
    let mut length_header = [0u8; 4];
    stream.as_ref().read_exact(&mut length_header)?;
    let length = u32::from_le_bytes(length_header);

    // Payload
    let mut buffer = vec![0u8; length as usize];
    stream.as_ref().read_exact(&mut buffer)?;
    stream.as_ref().flush()?;

    let packet = TcpPacket::try_from_bytes(&buffer)?;

    info!("Received packet: {:?}", packet);

    // Reserve the server state for this thread
    let mut server_state = match server_state.lock() {
        Ok(server_state) => server_state,
        Err(_) => {
            error!("Failed to lock the server state");
            return Ok(());
        }
    };

    match packet {
        TcpPacket::Connect(nickname) => {
            if let Err(e) = server_state.connect_user(stream.clone(), nickname) {
                let notification_packet = TcpPacket::Notification(e.to_string());
                let packet_bytes = notification_packet.to_bytes();
                stream.as_ref().write_all(&packet_bytes)?;
                stream.as_ref().flush()?;
                return Ok(());
            }

            let update_packet = TcpPacket::LoadCanvas(server_state.canvas.actions.clone());
            let packet_bytes = update_packet.to_bytes();

            let notification_packet = TcpPacket::Notification(format!("[+] {}", nickname));

            for connection in server_state.sessions.values_mut() {
                if connection.stream.as_ref().peer_addr()? != stream.peer_addr()? {
                    connection
                        .stream
                        .as_ref()
                        .write_all(&notification_packet.to_bytes())?;
                    connection.stream.as_ref().flush()?;
                }
            }

            stream.as_ref().write_all(&packet_bytes)?;
            stream.as_ref().flush()?;
        }

        TcpPacket::UpdateRequest(id, element) => {
            let previous_entry = server_state.canvas.get_entry(id).cloned();

            match server_state.canvas.update_entry(id, &element) {
                Some(entry) => {
                    let update_packet = TcpPacket::UpdateResponse(id, entry);
                    for connection in server_state.sessions.values_mut() {
                        let packet_bytes = update_packet.to_bytes();
                        connection.stream.as_ref().write_all(&packet_bytes)?;
                        connection.stream.as_ref().flush()?;
                    }

                    // Get the peer address from the stream
                    let peer_addr = stream.peer_addr()?;

                    // Get the connection associated with this peer address
                    let connection = server_state.sessions.get(&peer_addr);

                    // If there is a connection, get the user's nickname
                    let nickname = connection.and_then(|c| Some(c.nickname.as_str()));

                    // If there is a previous entry, proceed
                    if let Some(previous_entry) = previous_entry {
                        // If there is a nickname, get the user data associated with it
                        if let Some(nickname) = nickname {
                            if let Some(user_data) = server_state.users.get_mut(nickname) {
                                // Push the previous entry to the user's action history
                                user_data
                                    .action_history
                                    .push(Action::Update(previous_entry));
                            }
                        }
                    }
                }
                None => {
                    let notification_packet =
                        TcpPacket::Notification(format!("Entry with id {} does not exist", id));
                    let packet_bytes = notification_packet.to_bytes();
                    stream.as_ref().write_all(&packet_bytes)?;
                    stream.as_ref().flush()?;
                }
            }
        }

        TcpPacket::Delete(id) => {
            let entry = server_state.canvas.get_entry(id).cloned();

            if let Some(entry) = entry {
                let peer_addr = stream.peer_addr()?;
                let connection = server_state.sessions.get(&peer_addr);
                let nickname = connection.and_then(|c| Some(c.nickname.as_str()));

                if let Some(nickname) = nickname {
                    if let Some(user_data) = server_state.users.get_mut(nickname) {
                        user_data.action_history.push(Action::Delete(entry.clone()))
                    }
                }

                server_state.canvas.delete_entry(id);
                let update_packet = TcpPacket::Delete(id);
                for connection in server_state.sessions.values_mut() {
                    let packet_bytes = update_packet.to_bytes();
                    connection.stream.as_ref().write_all(&packet_bytes)?;
                    connection.stream.as_ref().flush()?;
                }
            }
        }

        TcpPacket::Undo => {
            let stream = stream.as_ref();
            let peer_addr = stream.peer_addr()?;
            let nickname = server_state
                .sessions
                .get(&peer_addr)
                .and_then(|c| Some(c.nickname.as_str()));

            let user = nickname.and_then(|nickname| server_state.users.get_mut(nickname));

            if let Some(user) = user {
                let last_action = user.action_history.pop();

                if let Some(last_action) = last_action {
                    match last_action {
                        Action::Delete(entry) => {
                            // Recreate entry
                            server_state.canvas.actions.push(entry.clone());
                            let draw_packet = TcpPacket::DrawResponse(entry);
                            for connection in server_state.sessions.values_mut() {
                                let bytes = draw_packet.to_bytes();
                                connection.stream.as_ref().write_all(&bytes)?;
                                connection.stream.as_ref().flush()?;
                            }
                        }
                        Action::Draw(id) => {
                            // Delete entry with that id
                            server_state.canvas.delete_entry(id);
                            let update_packet = TcpPacket::Delete(id);
                            for connection in server_state.sessions.values_mut() {
                                let packet_bytes = update_packet.to_bytes();
                                connection.stream.as_ref().write_all(&packet_bytes)?;
                                connection.stream.as_ref().flush()?;
                            }
                        }
                        Action::Update(previous_entry) => {
                            // Replace entry
                            let current_entry = server_state
                                .canvas
                                .actions
                                .iter_mut()
                                .find(|entry| entry.id == previous_entry.id);

                            if let Some(current_entry) = current_entry {
                                *current_entry = previous_entry
                            }
                        }
                        Action::Clear(prev_canvas_state) => {
                            //Replace the whole canvas
                            let actions = prev_canvas_state.actions.clone();
                            server_state.canvas = prev_canvas_state;

                            // Force all clients to full reload
                            for connection in server_state.sessions.values_mut() {
                                let update_packet = TcpPacket::LoadCanvas(actions.clone());
                                let packet_bytes = update_packet.to_bytes();
                                connection.stream.as_ref().write_all(&packet_bytes)?;
                                connection.stream.as_ref().flush()?;
                            }
                        }
                    }
                }
            }
        }

        TcpPacket::Disconnect => {
            server_state.disconnect_user(stream);
            return Ok(());
        }

        TcpPacket::DrawRequest(action) => {
            let peer_addr = stream.peer_addr()?;
            let nickname = server_state
                .sessions
                .get(&peer_addr)
                .and_then(|c| Some(c.nickname.as_str()));
            let user_data = nickname.and_then(|nickname| server_state.users.get(nickname));

            let new_entry_id = if let Some(user) = user_data {
                let new_entry = server_state.canvas.add_action(user.name.clone(), &action);

                // Send the update to all connected clients
                let update_packet = TcpPacket::DrawResponse(new_entry.clone());
                for connection in server_state.sessions.values_mut() {
                    let packet_bytes = update_packet.to_bytes();
                    connection.stream.as_ref().write_all(&packet_bytes)?;
                    connection.stream.as_ref().flush()?;
                }

                Some(new_entry.id)
            } else {
                None
            };

            // Add action to user history
            if let Some(user) = user_data {
                if let Some(id) = new_entry_id {
                    user.action_history.push(Action::Draw(id));
                }
            }
        }

        TcpPacket::ClearRequest { only_owned } => {
            // Save the previous state of the canvas
            let prev_canvas_state = server_state.canvas.clone();

            let peer_addr = stream.peer_addr()?;
            let nickname = server_state
                .sessions
                .get(&peer_addr)
                .and_then(|c| Some(c.nickname.as_str()));

            let user_data = nickname.and_then(|nickname| server_state.users.get_mut(nickname));

            // Put the clear action in the user history
            if let Some(user_data) = user_data {
                user_data
                    .action_history
                    .push(Action::Clear(prev_canvas_state));
            }

            // Decide which entries to delete
            let ids_to_delete = server_state
                .canvas
                .actions
                .iter()
                .filter_map(|entry| {
                    if only_owned {
                        if entry.author == nickname.unwrap() {
                            Some(entry.id)
                        } else {
                            None
                        }
                    } else {
                        Some(entry.id)
                    }
                })
                .collect();

            // Prepare the update packet
            let clear_packet = TcpPacket::ClearResponse { ids_to_delete };

            // Update all the clients
            for connection in server_state.sessions.values_mut() {
                let packet_bytes = clear_packet.to_bytes();
                connection.stream.as_ref().write_all(&packet_bytes)?;
                connection.stream.as_ref().flush()?;
            }
        }

        _ => {}
    }

    Ok(())
}

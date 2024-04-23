use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::Duration,
};

use ns_core::errors::{Error, Result, ServerError};
use ns_core::models::packets::TcpPacket;

use tracing::{debug, error, info};

use crate::models::{Action, ServerState, UserData};

pub fn handle_client(mut stream: TcpStream, server_state: Arc<Mutex<ServerState>>) -> Result<()> {
    // 10 minute timeout
    stream.set_read_timeout(Some(Duration::from_secs(600)))?;

    // Payload length
    let mut length_header = [0u8; 4];
    stream.read_exact(&mut length_header)?;
    let length = u32::from_le_bytes(length_header);

    // Payload
    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer)?;
    stream.flush()?;

    let packet = TcpPacket::try_from_bytes(&buffer)?;

    debug!("Received packet: {:?}", packet);

    let mut server_state = match server_state.lock() {
        Ok(server_state) => server_state,
        Err(_) => {
            error!("Failed to lock the server state");
            return Ok(());
        }
    };

    let mut users = server_state.users.clone();

    let user_data = server_state
        .get_username(&stream)
        .and_then(|name| users.get_mut(name));

    if let Some(user_data) = user_data {
        match packet {
            TcpPacket::DrawRequest(action) => {
                let new_entry_id = {
                    let new_entry = server_state
                        .canvas
                        .add_action(user_data.username.clone(), &action);

                    // Send the update to all connected clients
                    let update_packet = TcpPacket::DrawResponse(new_entry.clone());
                    for connection in server_state.sessions.iter_mut() {
                        let packet_bytes = update_packet.to_bytes()?;
                        connection.stream.write_all(&packet_bytes)?;
                        connection.stream.flush()?;
                    }

                    new_entry.id
                };

                info!(
                    "User {} drew {:?} at entry {}",
                    user_data.username, action, new_entry_id
                );

                // Add action to user history
                user_data.action_history.push(Action::Draw(new_entry_id));
            }

            TcpPacket::UpdateRequest(id, element) => {
                let previous_entry = server_state.canvas.get_entry(id).cloned();

                info!(
                    "User {} updated entry {} with {:?}",
                    user_data.username, id, element
                );

                match server_state.canvas.update_entry(id, &element) {
                    Some(entry) => {
                        let update_packet = TcpPacket::UpdateResponse(id, entry);
                        for connection in server_state.sessions.iter_mut() {
                            let packet_bytes = update_packet.to_bytes()?;
                            connection.stream.write_all(&packet_bytes)?;
                            connection.stream.flush()?;
                        }

                        if let Some(previous_entry) = previous_entry {
                            user_data
                                .action_history
                                .push(Action::Update(previous_entry))
                        }
                    }
                    None => {
                        let notification_packet =
                            TcpPacket::Notification(format!("Entry with id {} does not exist", id));
                        let packet_bytes = notification_packet.to_bytes()?;
                        stream.write_all(&packet_bytes)?;
                        stream.flush()?;
                    }
                }
            }

            TcpPacket::Delete(id) => {
                let entry = server_state.canvas.get_entry(id).cloned();

                info!("User {} deleted entry {}", user_data.username, id);

                if let Some(entry) = entry {
                    user_data.action_history.push(Action::Delete(entry.clone()));
                    server_state.canvas.delete_entry(id);
                    let update_packet = TcpPacket::Delete(id);
                    for connection in server_state.sessions.iter_mut() {
                        let packet_bytes = update_packet.to_bytes()?;
                        connection.stream.write_all(&packet_bytes)?;
                        connection.stream.flush()?;
                    }
                }
            }

            TcpPacket::Undo => {
                let last_action = user_data.action_history.pop();

                info!("User {} undid an action", user_data.username);

                if let Some(last_action) = last_action {
                    match last_action {
                        Action::Delete(entry) => {
                            // Recreate entry
                            server_state.canvas.entries.push(entry.clone());
                            let draw_packet = TcpPacket::DrawResponse(entry);
                            for connection in server_state.sessions.iter_mut() {
                                let bytes = draw_packet.to_bytes()?;
                                connection.stream.write_all(&bytes)?;
                                connection.stream.flush()?;
                            }
                        }
                        Action::Draw(id) => {
                            // Delete entry with that id
                            server_state.canvas.delete_entry(id);
                            let update_packet = TcpPacket::Delete(id);
                            for connection in server_state.sessions.iter_mut() {
                                let packet_bytes = update_packet.to_bytes()?;
                                connection.stream.write_all(&packet_bytes)?;
                                connection.stream.flush()?;
                            }
                        }
                        Action::Update(previous_entry) => {
                            // Replace entry
                            let current_entry = server_state
                                .canvas
                                .entries
                                .iter_mut()
                                .find(|entry| entry.id == previous_entry.id);

                            if let Some(current_entry) = current_entry {
                                *current_entry = previous_entry
                            }
                        }
                        Action::Clear(prev_canvas_state) => {
                            //Replace the whole canvas
                            let actions = prev_canvas_state.entries.clone();
                            server_state.canvas = prev_canvas_state;

                            // Force all clients to full reload
                            for connection in server_state.sessions.iter_mut() {
                                let update_packet = TcpPacket::LoadCanvas(actions.clone());
                                let packet_bytes = update_packet.to_bytes()?;
                                connection.stream.write_all(&packet_bytes)?;
                                connection.stream.flush()?;
                            }
                        }
                    }
                }
            }

            TcpPacket::Disconnect => {
                user_data.last_login = Some(std::time::Instant::now());
                server_state.users = users;
                // server_state.disconnect_user(&stream)?;
                return Ok(());
            }

            TcpPacket::ClearRequest { only_owned } => {
                // Save the previous state of the canvas
                let prev_canvas_state = server_state.canvas.clone();

                // Put the clear action in the user history
                user_data
                    .action_history
                    .push(Action::Clear(prev_canvas_state));

                // Decide which entries to delete
                let ids_to_delete = server_state
                    .canvas
                    .entries
                    .iter_mut()
                    .filter_map(|entry| {
                        if only_owned && entry.author != user_data.username {
                            return None;
                        }
                        Some(entry.id)
                    })
                    .collect();

                // Prepare the update packet
                let clear_packet = TcpPacket::ClearResponse { ids_to_delete };

                // Update all the clients
                for connection in server_state.sessions.iter_mut() {
                    let packet_bytes = clear_packet.to_bytes()?;
                    connection.stream.write_all(&packet_bytes)?;
                    connection.stream.flush()?;
                }
            }

            _ => {}
        }
    } else if let TcpPacket::Connect(nickname) = packet {
        if let Err(Error::ServerError(ServerError::UsernameTaken(s))) =
            server_state.connect_user(&stream, nickname.clone())
        {
            error!("Username {} is already connected", s);
            return Err(ServerError::UsernameTaken(s).into());
        }

        let update_packet = TcpPacket::LoadCanvas(server_state.canvas.entries.clone());
        let packet_bytes = update_packet.to_bytes()?;

        let notification_packet = TcpPacket::Notification(format!(
            "[+] {}",
            match server_state.get_username(&stream) {
                None => "Unknown",
                Some(s) => s,
            },
        ));

        let user = users
            .entry(nickname.clone())
            .or_insert(UserData::new(&nickname));

        match user.last_login {
            Some(last_login) => {
                let now = std::time::Instant::now();

                // TODO: Increase the time limit to one minute
                if now.duration_since(last_login).as_secs() > 5 {
                    info!("Clearing {}'s action history", nickname);
                    user.action_history.clear();
                }

                user.last_login = Some(now);
            }

            None => {
                info!("User {} connected", nickname);
                user.last_login = Some(std::time::Instant::now());
            }
        }

        for connection in server_state.sessions.iter_mut() {
            if connection.stream.peer_addr()? != stream.peer_addr()? {
                connection
                    .stream
                    .write_all(&notification_packet.to_bytes()?)?;
                connection.stream.flush()?;
            }
        }

        stream.write_all(&packet_bytes)?;
        stream.flush()?;
    } else {
        return Err(ServerError::UserNotFound.into());
    }

    server_state.users = users;

    // if let Err(e) = handle_client_inner(&mut server_state, stream.try_clone().unwrap()) {
    //     debug!("Unexpected logout: {:?}", e);
    //     server_state.disconnect_user(&stream).unwrap();
    //     break;
    // }

    Ok(())
}

// fn handle_client_inner(server_state: &mut ServerState, mut stream: TcpStream) -> Result<()>

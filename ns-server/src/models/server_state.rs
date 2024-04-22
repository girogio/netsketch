use std::{collections::HashMap, net::TcpStream};
use tracing::{error, warn};

use ns_core::errors::{Result, ServerError};
use ns_core::models::canvas::Canvas;

use super::{session::Session, user_data::UserData};

pub struct ServerState {
    pub canvas: Canvas,
    pub sessions: Vec<Session>,
    pub users: HashMap<String, UserData>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            canvas: Canvas::new(),
            sessions: Vec::new(),
            users: HashMap::new(),
        }
    }

    pub fn connect_user(&mut self, stream: &TcpStream, username: String) -> Result<()> {
        if self.sessions.iter().any(|x| x.username == username) {
            error!("Username {} is already connected", username);
            return Err(ServerError::UsernameTaken(username).into());
        } else {
            self.sessions
                .push(Session::new(stream.try_clone()?, username.clone()));
        }

        Ok(())
    }

    pub fn disconnect_user(&mut self, stream: &TcpStream) -> Result<()> {
        warn!("Ending session belonging to {:?}", stream.peer_addr()?);
        let peer_addr = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(e) => {
                error!("Failed to get peer address: {:?}", e);
                return Err(ServerError::UserNotFound.into());
            }
        };

        self.sessions
            .retain(|x| x.stream.peer_addr().ok() != Some(peer_addr));

        match stream.shutdown(std::net::Shutdown::Both) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to shutdown stream: {:?}", e);
                Err(ServerError::UserNotFound.into())
            }
        }
    }

    pub fn get_username(&self, stream: &TcpStream) -> Option<&String> {
        stream.peer_addr().ok().and_then(|addr| {
            self.sessions.iter().find_map(|session| {
                session.stream.peer_addr().ok().and_then(|peer_addr| {
                    if peer_addr == addr {
                        Some(&session.username)
                    } else {
                        None
                    }
                })
            })
        })
    }
}

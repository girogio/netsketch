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

    pub fn connect_user(&mut self, stream: &TcpStream, nickname: String) -> Result<()> {
        if self.sessions.iter().any(|x| x.username == nickname) {
            error!("Username {} is already connected", nickname);
            return Err(ServerError::UsernameTaken(nickname).into());
        } else {
            self.sessions
                .push(Session::new(stream.try_clone()?, nickname.clone()));
        }

        Ok(())
    }

    pub fn disconnect_user(&mut self, stream: &TcpStream) -> Result<()> {
        warn!("Disconnecting user: {:?}", stream.peer_addr()?);

        self.sessions
            .retain(|x| x.stream.peer_addr().ok() != stream.peer_addr().ok());

        stream.shutdown(std::net::Shutdown::Both)?;

        Ok(())
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

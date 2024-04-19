use std::{
    collections::HashMap,
    net::{TcpStream},
};

use ns_core::errors::{Result, ServerError};
use ns_core::models::canvas::Canvas;
use tracing::{error, warn};

use super::user_data::UserData;

pub struct Session {
    pub stream: TcpStream,
    pub username: String,
}

pub struct ServerState {
    pub canvas: Canvas,
    pub sessions: Vec<Session>,
    pub users: HashMap<String, UserData>,
}

impl Session {
    pub fn new(stream: TcpStream, username: String) -> Self {
        Session { stream, username }
    }
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
                .push(Session::new(stream.try_clone().unwrap(), nickname.clone()));
        }

        Ok(())
    }

    pub fn disconnect_user(&mut self, stream: &TcpStream) {
        warn!("Disconnecting user: {:?}", stream.peer_addr().unwrap());
        self.sessions
            .retain(|x| x.stream.peer_addr().unwrap() != stream.peer_addr().unwrap());
        stream.shutdown(std::net::Shutdown::Both).unwrap();
    }

    pub fn get_username(&self, stream: &TcpStream) -> Option<&String> {
        self.sessions
            .iter()
            .find(|x| x.stream.peer_addr().unwrap() == stream.peer_addr().unwrap())
            .map(|x| &x.username)
    }
}

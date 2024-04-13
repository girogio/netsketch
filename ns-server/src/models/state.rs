use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
    sync::Arc,
};
use tracing::{info, warn};

use ns_core::errors::{Result, ServerError};
use ns_core::models::canvas::Canvas;

use crate::models::user_data::UserData;

pub struct Session {
    pub nickname: String,
    pub stream: Arc<TcpStream>,
}

impl Session {
    pub fn new(nickname: String, stream: Arc<TcpStream>) -> Self {
        Session { nickname, stream }
    }
}

pub struct ServerState {
    pub canvas: Canvas,
    pub sessions: HashMap<SocketAddr, Session>,
    pub users: HashMap<String, UserData>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            canvas: Canvas::new(),
            sessions: HashMap::new(),
            users: HashMap::new(),
        }
    }

    pub fn connect_user(&mut self, stream: Arc<TcpStream>, nickname: String) -> Result<()> {
        if self
            .sessions
            .values()
            .any(|session| session.nickname == nickname)
        {
            warn!("User {} already connected", nickname);
            return Err(ServerError::UserAlreadyConnected(nickname).into());
        } else {
            self.sessions.insert(
                stream.peer_addr().unwrap(),
                Session::new(nickname.clone(), stream),
            );
        }

        let user = self
            .users
            .entry(nickname)
            .or_insert(UserData::new(&nickname));

        match user.last_login {
            None => {
                info!("New user connected: {:?}", stream.peer_addr().unwrap());
                user.last_login = Some(std::time::Instant::now());
            }

            Some(time) => {
                let now = std::time::Instant::now();
                if now.duration_since(time).as_secs() > 60 {
                    user.action_history.clear();
                } else {
                    user.last_login = Some(now);
                }
            }
        }

        Ok(())
    }

    pub fn disconnect_user(&mut self, stream: Arc<TcpStream>) {
        info!("Disconnecting user: {:?}", stream.peer_addr().unwrap());
        self.sessions.remove_entry(&stream.peer_addr().unwrap());
        stream.shutdown(std::net::Shutdown::Both).unwrap();
    }
}

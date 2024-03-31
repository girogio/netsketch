use std::{
    collections::HashMap,
    net::{SocketAddr, TcpStream},
};

use ns_core::models::canvas::Canvas;

pub struct ServerState {
    pub canvas: Canvas,
    pub connections: Vec<TcpStream>,
    pub users: HashMap<SocketAddr, String>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            canvas: Canvas::new(),
            connections: Vec::new(),
            users: HashMap::new(),
        }
    }

    pub fn connect_user(&mut self, stream: &TcpStream, nickname: String) {
        self.connections.push(stream.try_clone().unwrap());
        self.users.insert(stream.peer_addr().unwrap(), nickname);
    }

    pub fn disconnect_user(&mut self, stream: &TcpStream) {
        self.connections
            .retain(|x| x.peer_addr().unwrap() != stream.peer_addr().unwrap());
        self.users.remove(&stream.peer_addr().unwrap());
    }

    pub fn get_username(&self, stream: &TcpStream) -> String {
        self.users
            .get(&stream.peer_addr().unwrap())
            .unwrap_or(&"Unknown".to_string())
            .clone()
    }
}

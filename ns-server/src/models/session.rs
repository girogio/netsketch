use std::{net::TcpStream, sync::Arc};

#[derive(Clone)]
pub struct Session {
    pub nickname: String,
    pub stream: Arc<TcpStream>,
}

impl Session {
    pub fn new(nickname: String, stream: Arc<TcpStream>) -> Self {
        Session { nickname, stream }
    }
}

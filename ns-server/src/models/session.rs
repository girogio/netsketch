use std::net::TcpStream;

pub struct Session {
    pub stream: TcpStream,
    pub username: String,
}

impl Session {
    pub fn new(stream: TcpStream, username: String) -> Self {
        Session { stream, username }
    }
}

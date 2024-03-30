use crate::utils::errors::Result;
use std::net::{SocketAddr, TcpStream};

pub fn connect_to_server(server: &str, port: u16) -> Result<TcpStream> {
    let server = format!("{}:{}", server, port);
    let server: SocketAddr = server.parse()?;
    TcpStream::connect(server).map_err(|e| e.into())
}

use std::{io::Write, net::TcpStream, sync::mpsc::Sender};

use ns_core::errors::Result;
use ns_core::models::packets::Packet;

pub struct PacketSender;

impl PacketSender {
    pub fn start(address: String, port: u16) -> Result<Sender<Packet>> {
        let (tx, rx) = std::sync::mpsc::channel::<Packet>();
        let mut stream = TcpStream::connect(format!("{}:{}", address, port)).unwrap();

        std::thread::spawn(move || loop {
            match rx.recv() {
                Ok(packet) => {
                    let packet_bytes = packet.to_bytes();
                    stream.write_all(&packet_bytes).unwrap();
                }
                Err(e) => {
                    eprintln!("{e}");
                    break;
                }
            }
        });

        Ok(tx)
    }
}

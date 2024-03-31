use std::io::Read;
use std::{io::Write, net::TcpStream, sync::mpsc::Sender};

use ns_core::errors::Result;
use ns_core::models::canvas::CanvasEntry;
use ns_core::models::packets::Packet;

pub struct PacketHandler;

impl PacketHandler {
    pub fn start(
        address: String,
        port: u16,
        canvas_sender: Sender<CanvasEntry>,
    ) -> Result<Sender<Packet>> {
        let (tx, rx) = std::sync::mpsc::channel::<Packet>();
        let mut stream = TcpStream::connect(format!("{}:{}", address, port)).unwrap();

        let mut cloned_stream = stream.try_clone().unwrap();
        std::thread::spawn(move || loop {
            match rx.recv() {
                Ok(packet) => {
                    let packet_bytes = packet.to_bytes();
                    cloned_stream.write_all(&packet_bytes).unwrap();
                }
                Err(e) => {
                    eprintln!("{e}");
                    break;
                }
            }
        });

        std::thread::spawn(move || loop {
            let mut buffer = [0u8; 4];
            stream.read_exact(&mut buffer).unwrap();
            let length = u32::from_le_bytes(buffer);

            let mut buffer = vec![0u8; length as usize];
            stream.read_exact(&mut buffer).unwrap();

            let packet = Packet::try_from_bytes(&buffer);

            if let Ok(packet) = packet {
                match packet {
                    Packet::UpdatePeers(entry) => canvas_sender.send(entry).unwrap(),
                    Packet::LoadCanvas(entries) => {
                        for entry in entries {
                            canvas_sender.send(entry).unwrap();
                        }
                    }
                    Packet::Disconnect => {}
                    Packet::Connect(_) => {}
                    Packet::Draw(_) => (),
                }
            }
        });

        Ok(tx)
    }
}

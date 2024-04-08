use std::io::Read;
use std::{io::Write, net::TcpStream, sync::mpsc::Sender};

use ns_core::errors::{Error, Result};
use ns_core::models::packets::TcpPacket;

use crate::models::canvas::CanvasCommand;

pub struct TcpPacketHandler;

impl TcpPacketHandler {
    pub fn start(
        address: String,
        port: u16,
        canvas_sender: Sender<CanvasCommand>,
    ) -> Result<Sender<TcpPacket>> {
        // Connect to the server
        let mut stream = TcpStream::connect(format!("{}:{}", address, port))?;

        // Create a channel to send packets to the server
        let (tx, rx) = std::sync::mpsc::channel::<TcpPacket>();

        // Spawn a thread to send packets to the server
        let mut cloned_stream = stream.try_clone()?;
        std::thread::spawn(move || -> Result<()> {
            loop {
                match rx.recv() {
                    Ok(packet) => {
                        let packet_bytes = packet.to_bytes();
                        cloned_stream.write_all(&packet_bytes).unwrap();
                        cloned_stream.flush()?
                    }
                    Err(e) => {
                        eprintln!("{e}");
                    }
                }
            }
        });

        // Spawn a thread to receive packets from the server
        std::thread::spawn(move || loop {
            let mut task = || -> Result<()> {
                let packet = read_packet(&mut stream)?;

                match packet {
                    TcpPacket::DrawResponse(entry) => {
                        canvas_sender.send(CanvasCommand::Draw(entry)).unwrap();
                    }
                    TcpPacket::Notification(msg) => {
                        std::io::stdout().flush()?;
                        eprintln!("Notification: {}", msg);
                    }
                    TcpPacket::LoadCanvas(entries) => {
                        for entry in entries {
                            canvas_sender.send(CanvasCommand::Draw(entry)).unwrap();
                        }
                    }
                    TcpPacket::Delete(id) => {
                        canvas_sender.send(CanvasCommand::Delete(id)).unwrap();
                    }
                    TcpPacket::UpdateResponse(id, entry) => {
                        canvas_sender
                            .send(CanvasCommand::Overwrite(id, entry))
                            .unwrap();
                    }
                    TcpPacket::ClearResponse { ids_to_delete } => {
                        for id in ids_to_delete {
                            canvas_sender.send(CanvasCommand::Delete(id)).unwrap();
                        }
                    }
                    _ => {}
                }

                Ok(())
            };

            if let Err(Error::Io(e)) = task() {
                eprintln!("Disconnected from server. Actual error: {}", e);
                drop(stream);
                std::process::exit(1);
            }
        });

        Ok(tx)
    }
}

fn read_packet(stream: &mut TcpStream) -> Result<TcpPacket> {
    let mut length_header = [0u8; 4];
    stream.read_exact(&mut length_header)?;
    let length = u32::from_le_bytes(length_header);

    let mut buffer = vec![0u8; length as usize];
    stream.read_exact(&mut buffer)?;

    TcpPacket::try_from_bytes(&buffer)
}

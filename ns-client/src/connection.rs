use std::{io::Read, sync::Arc};
use std::{io::Write, net::TcpStream, sync::mpsc::Sender};

use ns_core::errors::{Error, Result};
use ns_core::models::packets::TcpPacket;

use crate::models::canvas::CanvasCommand;

pub struct TcpHandler;

impl TcpHandler {
    pub fn start(
        address: String,
        port: u16,
        canvas_sender: Sender<CanvasCommand>,
    ) -> Result<Sender<TcpPacket>> {
        // Connect to the server
        let stream = Arc::new(TcpStream::connect(format!("{}:{}", address, port))?);

        // Create a channel to send packets to the server
        let (tx, rx) = std::sync::mpsc::channel::<TcpPacket>();

        // Spawn a thread to send packets to the server
        let stream_ptr = stream.clone();
        std::thread::spawn(move || -> Result<()> {
            loop {
                match rx.recv() {
                    Ok(packet) => {
                        let packet_bytes = packet.to_bytes()?;
                        stream_ptr.as_ref().write_all(&packet_bytes)?;
                        stream_ptr.as_ref().flush()?
                    }
                    Err(e) => {
                        eprintln!("{e}");
                    }
                }
            }
        });

        // Spawn a thread to receive packets from the server
        std::thread::spawn(move || loop {
            let task = || -> Result<()> {
                let packet = read_packet(stream.clone())?;

                match packet {
                    TcpPacket::DrawResponse(entry) => {
                        canvas_sender.send(CanvasCommand::Draw(entry))?;
                    }

                    TcpPacket::Notification(msg) => {
                        std::io::stdout().flush()?;
                        println!("Notification: {}", msg);
                    }

                    TcpPacket::LoadCanvas(entries) => {
                        for entry in entries {
                            canvas_sender.send(CanvasCommand::Draw(entry))?;
                        }
                    }

                    TcpPacket::Delete(id) => {
                        canvas_sender.send(CanvasCommand::Delete(id))?;
                    }

                    TcpPacket::UpdateResponse(id, entry) => {
                        canvas_sender
                            .send(CanvasCommand::Overwrite(id, entry))
                            .unwrap();
                    }

                    TcpPacket::ClearResponse { ids_to_delete } => {
                        for id in ids_to_delete {
                            canvas_sender.send(CanvasCommand::Delete(id))?;
                        }
                    }
                    _ => {}
                }

                Ok(())
            };

            if let Err(Error::IoError(e)) = task() {
                eprintln!("Disconnected from server. Actual error: {}", e);
                drop(stream);
                std::process::exit(1);
            }
        });

        Ok(tx)
    }
}

fn read_packet(stream: Arc<TcpStream>) -> Result<TcpPacket> {
    let mut length_header = [0u8; 4];
    stream.as_ref().read_exact(&mut length_header)?;
    let length = u32::from_le_bytes(length_header);

    let mut buffer = vec![0u8; length as usize];
    stream.as_ref().read_exact(&mut buffer)?;

    TcpPacket::try_from_bytes(&buffer)
}

use bincode::{config, Decode, Encode};

use super::commands::CanvasCommand;

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum Packet {
    Connect(String),
    Disconnect(String),
    Command(CanvasCommand),
}

impl Packet {
    pub fn to_bytes(&self) -> Vec<u8> {
        let packet_data = bincode::encode_to_vec(self, config::standard()).unwrap();
        let mut packet_length_bytes = (packet_data.len() as u32).to_le_bytes().to_vec();
        packet_length_bytes.extend(packet_data);
        packet_length_bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        bincode::decode_from_slice(bytes, config::standard())
            .unwrap()
            .0
    }
}

impl std::fmt::Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Packet::Connect(nickname) => write!(f, "Connect: {}", nickname),
            Packet::Disconnect(nickname) => write!(f, "Disconnect: {}", nickname),
            Packet::Command(command) => write!(f, "Command: {}", command),
        }
    }
}

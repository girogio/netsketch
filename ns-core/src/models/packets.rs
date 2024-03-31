use crate::{
    errors::Result,
    models::{canvas::CanvasEntry, commands::CanvasElement},
};

use bincode::{config, Decode, Encode};

#[derive(Encode, Decode, Debug, Clone)]
pub enum Packet {
    /// Sent by the client to the server when the user wants to connect to the server.
    Connect(String),
    /// Sent by the client to the server when the user wants to disconnect from the server.
    Disconnect,
    /// Sent by the client to the server when the user wants to draw something on the canvas.
    Draw(CanvasElement),
    /// Sent by the server to the clients when the server wants to update the client's canvas.
    UpdatePeers(CanvasEntry),
    /// Sent by the server to the clients when the server wants to load the entire canvas.
    LoadCanvas(Vec<CanvasEntry>),
}

impl Packet {
    /// Helper function to convert the packet to a byte vector.\
    /// Datagram:
    /// ```plaintext
    /// | 4 bytes | n bytes |
    /// | length  | data    |
    /// ```
    /// where `length` is the length of the `data` field.
    ///
    /// The `length` field is a little-endian u32. \
    /// The `data` field is the encoded packet, which is a [Packet] enum.
    pub fn to_bytes(&self) -> Vec<u8> {
        let packet_data = bincode::encode_to_vec(self, config::standard()).unwrap();
        let mut packet_length_bytes = (packet_data.len() as u32).to_le_bytes().to_vec();
        packet_length_bytes.extend(packet_data);
        packet_length_bytes
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bincode::decode_from_slice(bytes, config::standard())?.0)
    }
}

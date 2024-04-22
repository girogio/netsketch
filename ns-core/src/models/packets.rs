use std::io::Write;

use crate::{
    errors::Result,
    models::canvas::{CanvasElement, CanvasEntry},
};

use bincode::{config, Decode, Encode};

#[derive(Encode, Decode, Debug, Clone)]
pub enum TcpPacket {
    /// Sent by the client to the server when the user wants to connect to the server.
    Connect(String),
    /// Sent by the client to the server when the user wants to disconnect from the server.
    Disconnect,
    /// Sent by the client to the server when the user wants to draw something on the canvas.
    DrawRequest(CanvasElement),
    /// Sent by the server to the clients when the server wants to update the client's canvas.
    DrawResponse(CanvasEntry),
    /// Sent by the client to the server when the user wants to delete an element from the canvas.
    /// Also sent by the server to the clients as an update to the canvas.
    Delete(usize),
    /// Sent by the client to the server when the user wants to clear the canvas.
    /// The boolean is true if the client requested for a full clear
    ClearRequest { only_owned: bool },
    /// Sent by the server to the clients when the server wants to clear the canvas.
    ClearResponse { ids_to_delete: Vec<usize> },
    /// Sent by the client to the server when the user wants to update an entry on the canvas.
    UpdateRequest(usize, CanvasElement),
    /// Sent by the server to the clients when the server wants to update a specific entry on the canvas.
    UpdateResponse(usize, CanvasEntry),
    /// Sent by the server to the clients when the server wants to load the entire canvas at the beginning.
    LoadCanvas(Vec<CanvasEntry>),
    /// Sent by the server to the client when the server wants to notify the client of something.
    Notification(String),
    /// Sent by the client to the server when the user wants to undo an action.
    Undo,
}

impl TcpPacket {
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
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let payload = bincode::encode_to_vec(self, config::standard())?;
        let length = (payload.len() as u32).to_le_bytes().to_vec();
        let packet = [length, payload].concat();

        Ok(packet)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bincode::decode_from_slice(bytes, config::standard())?.0)
    }
}

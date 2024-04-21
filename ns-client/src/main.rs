mod connection;
mod models;
mod operations;

use std::thread::spawn;

use clap::Parser;

use crate::{
    connection::TcpPacketHandler,
    models::canvas::{CanvasCommand, ClientCanvas},
    operations::handle_prompt,
};

use ns_core::errors::Result;
use ns_core::models::packets::TcpPacket;

#[derive(Parser)]
#[command(version, about, author)]
struct Cli {
    #[clap(short, long)]
    /// The address of the netsketch server
    address: String,
    #[clap(short, long)]
    /// The port of the netsketch server
    port: u16,
    #[clap(short, long)]
    /// The nickname of the user
    nickname: String,
}

#[macroquad::main("NetSketch")]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let (canvas_sender, canvas_receiver) = std::sync::mpsc::channel::<CanvasCommand>();

    let tcp_handler =
        TcpPacketHandler::start(args.address.to_string(), args.port, canvas_sender.clone())?;

    tcp_handler.send(TcpPacket::Connect(args.nickname.clone()))?;

    println!("Connected to server at {}:{}\n", args.address, args.port);

    let tx_cloned = tcp_handler.clone();

    spawn(move || handle_prompt(tx_cloned, canvas_sender));

    let mut canvas = ClientCanvas::new(args.nickname, canvas_receiver, tcp_handler);

    loop {
        canvas.draw().await;
    }
}

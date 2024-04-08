mod cli;
mod connection;
mod models;

use clap::Parser;
use macroquad::{
    prelude::*,
};

use crate::{
    connection::TcpPacketHandler,
    models::canvas::{CanvasCommand, ClientCanvas},
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
    prevent_quit();

    let args = Cli::parse();

    let (canvas_sender, canvas_receiver) = std::sync::mpsc::channel::<CanvasCommand>();

    let tcp_handler =
        TcpPacketHandler::start(args.address.to_string(), args.port, canvas_sender.clone())?;

    tcp_handler
        .send(TcpPacket::Connect(args.nickname.clone()))
        .expect("Failed to connect to server.");

    println!("Connected to server at {}:{}", args.address, args.port,);
    println!();

    let tx_cloned = tcp_handler.clone();

    std::thread::spawn(move || cli::handle_ns_prompt(tx_cloned, canvas_sender));

    let mut canvas = ClientCanvas::new(args.nickname, canvas_receiver, tcp_handler);

    loop {
        canvas.draw().await;
    }
}

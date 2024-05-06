mod connection;
mod models;
mod operations;

use clap::Parser;
use macroquad::window::{clear_background, next_frame, Conf};
use std::thread::spawn;

use crate::{
    connection::TcpHandler,
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

fn window_conf() -> Conf {
    Conf {
        window_title: "Netsketch".to_owned(),
        high_dpi: true,
        sample_count: 100,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<()> {
    let args = match Cli::try_parse() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{}", err);
            clear_background([0.0, 0.0, 0.0, 1.0].into());
            next_frame().await;
            std::process::exit(1);
        }
    };

    let (canvas_sender, canvas_receiver) = std::sync::mpsc::channel::<CanvasCommand>();

    let tcp_handler =
        TcpHandler::start(args.address.to_string(), args.port, canvas_sender.clone())?;

    tcp_handler.send(TcpPacket::Connect(args.nickname.clone()))?;

    println!("Connected to server at {}:{}\n", args.address, args.port);

    let tx_cloned = tcp_handler.clone();

    spawn(move || handle_prompt(tx_cloned, canvas_sender));

    let mut canvas = ClientCanvas::new(args.nickname, canvas_receiver, tcp_handler);

    canvas.draw().await;

    Ok(())
}

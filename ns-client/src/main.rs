mod connection;
mod utils;

use std::{io::Write, net::TcpStream};

use clap::Parser;
use connection::connect_to_server;
use macroquad::{
    // audio::{self, PlaySoundParams},
    prelude::*,
    ui::{hash, root_ui, widgets::Window},
};
use ns_core::models::packets::Packet;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    /// The address of the netsketch server
    server: String,
    #[clap(short, long)]
    /// The port of the netsketch server
    port: u16,
    #[clap(short, long)]
    /// The nickname of the user
    nickname: String,
}

#[macroquad::main("NetSketch")]
async fn main() {
    prevent_quit();

    let args = Cli::parse();

    let mut tcp_stream = match connect_to_server(&args.server, args.port) {
        Ok(tcp_stream) => tcp_stream,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    }; // this stream is closed when the variable tcp_stream is dropped

    let packet_bytes = Packet::Connect(args.nickname).to_bytes();
    tcp_stream.write_all(&packet_bytes).unwrap();

    draw_game_canvas(&mut tcp_stream).await;
}

async fn draw_game_canvas(tcp_stream: &mut TcpStream) {
    set_pc_assets_folder("ns-client/assets");

    // let bg_music = audio::load_sound("bg_music.wav").await.unwrap();

    let mut show_exit_dialog = false;
    let mut user_decided_to_exit = false;

    // audio::play_sound(
    //     &bg_music,
    //     PlaySoundParams {
    //         volume: 0.1,
    //         looped: true,
    //     },
    // );

    loop {
        clear_background(LIGHTGRAY);

        if is_quit_requested() || is_key_down(KeyCode::Escape) {
            show_exit_dialog = true;
        }

        draw_circle(15.0, 15.0, 15.0, YELLOW);

        if show_exit_dialog {
            draw_exit_dialog(&mut user_decided_to_exit, &mut show_exit_dialog, tcp_stream);
        }

        if user_decided_to_exit {
            break;
        }

        next_frame().await;
    }
}

fn draw_exit_dialog(
    user_decided_to_exit: &mut bool,
    show_exit_dialog: &mut bool,
    tcp_stream: &mut TcpStream,
) {
    let dialog_size = vec2(200., 70.);
    let screen_size = vec2(screen_width(), screen_height());
    let dialog_position = screen_size / 2. - dialog_size / 2.;
    Window::new(hash!(), dialog_position, dialog_size).ui(&mut root_ui(), |ui| {
        ui.label(None, "Do you really want to quit?");
        ui.separator();
        ui.same_line(60.);
        if ui.button(None, "Yes") {
            let packet_bytes = Packet::Disconnect("".to_string()).to_bytes();
            tcp_stream.write_all(&packet_bytes).unwrap();
            *user_decided_to_exit = true;
        }
        ui.same_line(120.);
        if ui.button(None, "No") {
            *show_exit_dialog = false;
        }
    });
}

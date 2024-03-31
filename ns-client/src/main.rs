mod cli;
mod connection;

use std::sync::mpsc::{Receiver, Sender};

use clap::Parser;
use macroquad::{
    audio::{self, PlaySoundParams},
    prelude::*,
    ui::{hash, root_ui, widgets::Window},
};
use ns_core::models::{
    canvas::{Canvas, CanvasEntry},
    packets::Packet,
};

use crate::connection::PacketHandler;

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

    let (canvas_sender, canvas_receiver) = std::sync::mpsc::channel::<CanvasEntry>();

    let packet_sender = PacketHandler::start(args.server.to_string(), args.port, canvas_sender)
        .expect("Failed to start packet handler");

    println!("{}", "=".repeat(50));
    println!("Connected to server at {}:{}", args.server, args.port);
    println!("{}", "=".repeat(50));

    packet_sender.send(Packet::Connect(args.nickname)).unwrap();

    let tx_cloned = packet_sender.clone();

    std::thread::spawn(move || cli::handle_ns_prompt(tx_cloned));

    draw_game_canvas(packet_sender.clone(), canvas_receiver).await;
}

async fn draw_game_canvas(tx: Sender<Packet>, canvas_receiver: Receiver<CanvasEntry>) {
    set_pc_assets_folder("ns-client/assets/music");

    let bg_music = audio::load_sound("music.wav").await.unwrap();

    let mut show_exit_dialog = false;
    let mut user_decided_to_exit = false;

    audio::play_sound(
        &bg_music,
        PlaySoundParams {
            volume: 0.1,
            looped: true,
        },
    );

    let mut canvas = Canvas::new();

    loop {
        clear_background(LIGHTGRAY);

        if is_quit_requested() || is_key_down(KeyCode::Escape) {
            show_exit_dialog = true;
        }

        if show_exit_dialog {
            draw_exit_dialog(&mut user_decided_to_exit, &mut show_exit_dialog);
        }

        if user_decided_to_exit {
            tx.send(Packet::Disconnect).unwrap();
            break;
        }

        while let Ok(command) = canvas_receiver.try_recv() {
            canvas.actions.push(command.clone());
        }

        canvas.draw();

        next_frame().await;
    }
}

fn draw_exit_dialog(user_decided_to_exit: &mut bool, show_exit_dialog: &mut bool) {
    let dialog_size = vec2(200., 70.);
    let screen_size = vec2(screen_width(), screen_height());
    let dialog_position = screen_size / 2. - dialog_size / 2.;
    Window::new(hash!(), dialog_position, dialog_size).ui(&mut root_ui(), |ui| {
        ui.label(None, "Do you really want to quit?");
        ui.separator();
        ui.same_line(60.);
        if ui.button(None, "Yes") {
            *user_decided_to_exit = true;
        }
        ui.same_line(120.);
        if ui.button(None, "No") {
            *show_exit_dialog = false;
        }
    });
}

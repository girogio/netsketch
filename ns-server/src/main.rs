mod models;
mod operations;

use clap::Parser;
use std::{
    process::exit,
    sync::{Arc, Mutex},
};
use tracing::error;

use models::ServerState;
use operations::{handle_client, init_server};

#[derive(Parser)]
struct Args {
    /// The address of the netsketch server
    #[clap(short, long)]
    address: String,
    /// The port of the netsketch server
    #[clap(short, long)]
    port: u16,
}

fn main() {
    let server = match init_server(Args::parse()) {
        Ok(server_state) => server_state,
        Err(e) => {
            error!("{e}", e = e);
            exit(1);
        }
    };

    let server_state = Arc::new(Mutex::new(ServerState::new()));

    for stream in server.incoming() {
        let server_state = server_state.clone();
        match stream {
            Ok(stream) => handle_client(stream, server_state),
            Err(e) => {
                error!("{e}", e = e.kind());
            }
        }
    }
}

use std::{net::TcpListener, process::exit};

use crate::Args;
use ns_core::errors::Result;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

pub fn init_server(args: Args) -> Result<TcpListener> {
    // let args = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let server = match TcpListener::bind(format!("{}:{}", args.address, args.port)) {
        Ok(server) => server,
        Err(e) => {
            error!(
                "Failed to bind the server to {}:{}\nReason: {}",
                args.address, args.port, e
            );
            exit(1);
        }
    };

    info!(
        "Listening for connections on {}:{}...",
        args.address, args.port
    );

    Ok(server)
}

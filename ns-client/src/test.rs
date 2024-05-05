mod connection;
mod models;
mod operations;

use clap::Parser;

use crate::{connection::TcpPacketHandler, models::canvas::CanvasCommand};

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

fn main() -> Result<()> {
    use models::enums::ToolType;
    use ns_core::models::canvas::CanvasElement;
    use rand::Rng;

    let args = Cli::parse();

    let (canvas_sender, _a) = std::sync::mpsc::channel::<CanvasCommand>();

    let tcp_handler =
        TcpPacketHandler::start(args.address.to_string(), args.port, canvas_sender.clone())?;

    tcp_handler.send(TcpPacket::Connect(args.nickname.clone()))?;

    let tcp_handler_c = tcp_handler.clone();

    let count = 20;
    let upper = 500;
    let mut random = rand::thread_rng();

    // 4 random points
    for _ in 0..count {
        let (x1, x2, y1, y2) = (
            random.gen_range(200..upper) as u16,
            random.gen_range(200..upper) as u16,
            random.gen_range(200..upper) as u16,
            random.gen_range(200..upper) as u16,
        );

        let colour: [u8; 4] = [
            random.gen_range(0..255),
            random.gen_range(0..255),
            random.gen_range(0..255),
            255,
        ];

        let tool = match random.gen_range(0..3) {
            0 => ToolType::Line,
            1 => ToolType::Rectangle,
            2 => ToolType::Circle,
            _ => ToolType::Text,
        };

        let element = match tool {
            ToolType::Line => CanvasElement::Line {
                x1,
                y1,
                x2,
                y2,
                colour,
            },
            ToolType::Rectangle => CanvasElement::Rect {
                x: x1,
                y: y1,
                width: x2,
                height: y2,
                colour,
            },
            ToolType::Circle => CanvasElement::Circle {
                x: x1,
                y: y1,
                radius: x2,
                colour,
            },
            ToolType::Text => CanvasElement::Text {
                x: x1,
                y: y1,
                text: "Hello, World!".to_string(),
                colour,
            },
        };

        let packet = TcpPacket::DrawRequest(element);

        tcp_handler.send(packet)?;

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    tcp_handler_c.send(TcpPacket::Disconnect)?;

    Ok(())
}

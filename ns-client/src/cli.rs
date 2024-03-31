use std::{io::Write, sync::mpsc::Sender};

use ns_core::models::{commands::CanvasElement, packets::Packet};

enum Tool {
    Line,
    Circle,
    Rectangle,
    Text,
}

pub fn handle_ns_prompt(packet_sender: Sender<Packet>) {
    let stdin = std::io::stdin();

    let mut colour: [u8; 4] = [0, 0, 0, 255];
    let mut tool = Tool::Line;

    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut buffer = String::new();
        stdin.read_line(&mut buffer).unwrap();
        let args = buffer.split_whitespace().collect::<Vec<&str>>();
        match args.as_slice() {
            ["draw", ..] => match tool {
                Tool::Line => {
                    println!("Drawing line");
                    let x1 = args[1].parse().unwrap();
                    let y1 = args[2].parse().unwrap();
                    let x2 = args[3].parse().unwrap();
                    let y2 = args[4].parse().unwrap();

                    let packet = Packet::Draw(CanvasElement::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        colour,
                    });

                    packet_sender.send(packet).unwrap();
                }
                Tool::Circle => {
                    println!("Drawing circle");
                    let x = args[1].parse().unwrap();
                    let y = args[2].parse().unwrap();
                    let radius = args[3].parse().unwrap();

                    let packet = Packet::Draw(CanvasElement::Circle {
                        x,
                        y,
                        radius,
                        colour,
                    });

                    packet_sender.send(packet).unwrap();
                }
                Tool::Rectangle => {
                    println!("Drawing rectangle");

                    let x = args[1].parse().unwrap();
                    let y = args[2].parse().unwrap();
                    let width = args[3].parse().unwrap();
                    let height = args[4].parse().unwrap();

                    let packet = Packet::Draw(CanvasElement::Rect {
                        x,
                        y,
                        width,
                        height,
                        colour,
                    });

                    packet_sender.send(packet).unwrap();
                }
                Tool::Text => {
                    println!("Drawing text");

                    let x = args[1].parse().unwrap();
                    let y = args[2].parse().unwrap();
                    let text = args[3..].join(" ");

                    let packet = Packet::Draw(CanvasElement::Text { x, y, text, colour });

                    packet_sender.send(packet).unwrap();
                }
            },

            ["colour", r, g, b, a] => {
                println!("Changing colour to ({}, {}, {}, {})", r, g, b, a);
                colour = [
                    r.parse().unwrap(),
                    g.parse().unwrap(),
                    b.parse().unwrap(),
                    a.parse().unwrap(),
                ];
            }

            ["tool", _] => {
                tool = match args[1] {
                    "line" => Tool::Line,
                    "circle" => Tool::Circle,
                    "rectangle" => Tool::Rectangle,
                    "text" => Tool::Text,
                    _ => {
                        eprintln!("Invalid tool");
                        continue;
                    }
                }
            }

            ["clear", "all" | "mine"] => {
                println!("Clearing canvas");
            }

            ["exit"] => {
                packet_sender.send(Packet::Disconnect).unwrap();
                drop(packet_sender);
                std::process::exit(0);
            }

            _ => {
                eprintln!("Invalid command");
            }
        }
    }
}

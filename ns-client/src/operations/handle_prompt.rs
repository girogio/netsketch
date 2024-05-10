use std::{io::Write, sync::mpsc::Sender};

use ns_core::errors::Result;
use ns_core::models::{canvas::CanvasElement, packets::TcpPacket};

use crate::models::canvas::CanvasCommand;
use crate::models::enums::{Filter, Ownership, ToolType};

pub fn handle_prompt(
    packet_sender: Sender<TcpPacket>,
    canvas_sender: Sender<CanvasCommand>,
) -> Result<()> {
    let stdin = std::io::stdin();

    let mut colour: [u8; 4] = [0, 0, 0, 255];
    let mut tool = ToolType::Line;
    let mut selected_id: Option<usize> = None;

    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;
        let args = buffer.split_whitespace().collect::<Vec<&str>>();
        match args.as_slice() {
            ["draw", ..] => {
                let element = match tool {
                    ToolType::Line => {
                        println!("Drawing line");
                        let x1 = args[1].parse()?;
                        let y1 = args[2].parse()?;
                        let x2 = args[3].parse()?;
                        let y2 = args[4].parse()?;

                        CanvasElement::Line {
                            x1,
                            y1,
                            x2,
                            y2,
                            colour,
                        }
                    }
                    ToolType::Circle => {
                        println!("Drawing circle");
                        let x = args[1].parse().unwrap();
                        let y = args[2].parse().unwrap();
                        let radius = args[3].parse().unwrap();

                        CanvasElement::Circle {
                            x,
                            y,
                            radius,
                            colour,
                        }
                    }
                    ToolType::Rectangle => {
                        println!("Drawing rectangle");

                        let x = args[1].parse().unwrap();
                        let y = args[2].parse().unwrap();
                        let width = args[3].parse().unwrap();
                        let height = args[4].parse().unwrap();

                        CanvasElement::Rect {
                            x,
                            y,
                            width,
                            height,
                            colour,
                        }
                    }
                    ToolType::Text => {
                        println!("Drawing text");

                        let x = args[1].parse().unwrap();
                        let y = args[2].parse().unwrap();
                        let text = args[3..].join(" ");

                        CanvasElement::Text { x, y, text, colour }
                    }
                };

                let packet = match selected_id {
                    Some(id) => TcpPacket::UpdateRequest(id, element),
                    None => TcpPacket::DrawRequest(element),
                };
                selected_id = None;

                packet_sender.send(packet).unwrap();
            }

            ["colour", r, g, b, a] => {
                println!("Changing colour to ({}, {}, {}, {})", r, g, b, a);
                colour = [
                    r.parse().unwrap(),
                    g.parse().unwrap(),
                    b.parse().unwrap(),
                    a.parse().unwrap(),
                ];

                canvas_sender
                    .send(CanvasCommand::ChangeColour(colour))
                    .unwrap();
            }

            ["tool", _] => {
                tool = match args[1] {
                    "line" => ToolType::Line,
                    "circle" => ToolType::Circle,
                    "rectangle" => ToolType::Rectangle,
                    "text" => ToolType::Text,
                    _ => {
                        eprintln!("Invalid tool");
                        continue;
                    }
                };
                canvas_sender.send(CanvasCommand::ChangeTool(tool)).unwrap();
            }

            ["select", _] => match args[1] {
                "none" => {
                    println!("Deselecting element");
                    selected_id = None;
                }
                _ => {
                    selected_id = Some(args[1].parse()?);
                }
            },

            ["show", "all" | "mine"] => match args[1] {
                "all" => {
                    canvas_sender.send(CanvasCommand::ShowAll).unwrap();
                }
                "mine" => {
                    canvas_sender.send(CanvasCommand::ShowMine).unwrap();
                }
                _ => unreachable!(),
            },

            ["delete", _] => {
                let id: usize = args[1].parse()?;

                packet_sender.send(TcpPacket::Delete(id)).unwrap();
            }

            ["list", "all" | "line" | "rect" | "circle" | "text", "all" | "mine"] => {
                let filter = Filter {
                    tool_type: match args[1] {
                        "all" => None,
                        "line" => Some(ToolType::Line),
                        "circle" => Some(ToolType::Circle),
                        "text" => Some(ToolType::Text),
                        _ => unreachable!(),
                    },
                    ownership: match args[2] {
                        "all" => Ownership::All,
                        "mine" => Ownership::Mine,
                        _ => unreachable!(),
                    },
                };

                canvas_sender.send(CanvasCommand::List(filter)).unwrap();
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            ["clear", "all" | "mine"] => {
                packet_sender
                    .send(TcpPacket::ClearRequest {
                        only_owned: match args[1] {
                            "all" => false,
                            "mine" => true,
                            _ => unreachable!(),
                        },
                    })
                    .unwrap();
            }

            ["undo"] => packet_sender.send(TcpPacket::Undo).unwrap(),

            ["help"] => {
                println!("Commands:");
                println!("draw <args> - Draw an element on the canvas");
                println!("  if tool = line:        <x1> <y1> <x2> <y2> - Draw a line");
                println!("  if tool = circle:      <x> <y> <radius> - Draw a circle");
                println!("  if tool = rectangle:   <x> <y> <width> <height> - Draw a rectangle");
                println!("  if tool = text:        <x> <y> <text> - Draw text");
                println!("colour <r> <g> <b> <a> - Change the colour of the element");
                println!("tool < line | circle | rectangle | text > - Change the tool");
                println!("select < id > - Select an element by id");
                println!("show < all | mine > - Show all elements or only your own");
                println!("delete < id > - Delete an element by id");
                println!(
                    "list < all | line | rect | circle | text > < all | mine > - List elements"
                );
                println!("clear < all | mine > - Clear all elements or only your own");
                println!("undo - Undo the last action");
                println!("exit - Exit the program");
            }

            ["exit"] => {
                packet_sender.send(TcpPacket::Disconnect).unwrap();
                drop(canvas_sender);
                drop(packet_sender);
                std::process::exit(0);
            }

            _ => {
                eprintln!("Invalid command");
            }
        }
    }
}

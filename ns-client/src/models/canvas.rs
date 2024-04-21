use std::sync::mpsc::{Receiver, Sender};

use macroquad::{
    color::LIGHTGRAY,
    input::{is_key_down, is_quit_requested, prevent_quit, KeyCode},
    math::vec2,
    shapes::{draw_circle, draw_line, draw_rectangle},
    text::draw_text,
    ui::{hash, root_ui, widgets::Window},
    window::{clear_background, next_frame, screen_height, screen_width},
};
use ns_core::models::{
    canvas::{Canvas, CanvasElement, CanvasEntry},
    packets::TcpPacket,
};

use super::enums::{Filter, Ownership, ToolType};

pub struct ClientCanvas {
    pub nickname: String,
    pub canvas: Canvas,
    pub selected_tool: ToolType,
    pub selected_colour: [u8; 4],
    pub user_decided_to_exit: bool,
    pub show_exit_dialog: bool,
    pub canvas_receiver: Receiver<CanvasCommand>,
    pub tcp_packet_sender: Sender<TcpPacket>,
}

#[derive(Debug, Clone)]
pub enum CanvasCommand {
    Draw(CanvasEntry),
    Delete(usize),
    Overwrite(usize, CanvasEntry),
    List(Filter),
    ChangeTool(ToolType),
    ChangeColour([u8; 4]),
    ShowAll,
    ShowMine,
}

impl ClientCanvas {
    pub fn new(
        nickname: String,
        canvas_receiver: Receiver<CanvasCommand>,
        tcp_packet_sender: Sender<TcpPacket>,
    ) -> Self {
        Self {
            nickname,
            selected_tool: ToolType::Line,
            selected_colour: [0, 0, 0, 255],
            canvas: Canvas::new(),
            user_decided_to_exit: false,
            show_exit_dialog: false,
            canvas_receiver,
            tcp_packet_sender,
        }
    }

    pub fn process_command(&mut self, command: CanvasCommand) {
        match command.clone() {
            CanvasCommand::ShowAll => {
                self.canvas
                    .actions
                    .iter_mut()
                    .for_each(|entry| entry.shown = true);
            }

            CanvasCommand::ShowMine => {
                self.canvas
                    .actions
                    .iter_mut()
                    .for_each(|entry| entry.shown = entry.author == self.nickname);
            }

            CanvasCommand::Draw(entry) => self.canvas.actions.push(entry),

            CanvasCommand::Overwrite(id, new_entry) => {
                if self.canvas.update_entry(id, &new_entry.element).is_none() {
                    println!("Entry with id {} does not exist", id);
                }
            }

            CanvasCommand::List(filter) => {
                self
                    .canvas.actions
                            .iter()
                            .filter(|entry| match filter.tool_type {
                                Some(ToolType::Line) => {
                                    matches!(entry.element, CanvasElement::Line { .. })
                                }
                                Some(ToolType::Circle) => {
                                    matches!(entry.element, CanvasElement::Circle { .. })
                                }
                                Some(ToolType::Rectangle) => {
                                    matches!(entry.element, CanvasElement::Rect { .. })
                                }
                                Some(ToolType::Text) => {
                                    matches!(entry.element, CanvasElement::Text { .. })
                                }
                                None => true,
                            } && match filter.ownership {
                                Ownership::All => true,
                                Ownership::Mine => entry.author == self.nickname,
                            }
                            ).for_each(|entry| {
                                println!("{}", entry);
                            });
            }

            CanvasCommand::Delete(id) => self.canvas.delete_entry(id),

            CanvasCommand::ChangeTool(tool) => self.selected_tool = tool,

            CanvasCommand::ChangeColour(colour) => {
                self.selected_colour = colour;
            }
        }
    }

    /// This function should only be called in the same thread where the canvas
    /// provided by [`macroquad`] is being drawn.
    fn draw_action(&self, entry: &CanvasEntry) {
        match &entry.element {
            CanvasElement::Line {
                x1,
                y1,
                x2,
                y2,
                colour,
            } => {
                draw_line(
                    *x1 as f32,
                    *y1 as f32,
                    *x2 as f32,
                    *y2 as f32,
                    5.,
                    (*colour).into(),
                );
            }
            CanvasElement::Circle {
                x,
                y,
                radius,
                colour,
            } => {
                draw_circle(*x as f32, *y as f32, *radius as f32, (*colour).into());
            }
            CanvasElement::Rect {
                x,
                y,
                width,
                height,
                colour,
            } => {
                draw_rectangle(
                    *x as f32,
                    *y as f32,
                    *width as f32,
                    *height as f32,
                    (*colour).into(),
                );
            }
            CanvasElement::Text { x, y, text, colour } => {
                draw_text(text, *x as f32, *y as f32, 50., (*colour).into());
            }
        }
    }

    /// This function should only be called in the same thread where the canvas
    /// provided by [`macroquad`] is being drawn.
    pub async fn draw(&mut self) {
        prevent_quit();
        loop {
            clear_background(LIGHTGRAY);

            let tool_icon = match self.selected_tool {
                ToolType::Line => "line",
                ToolType::Circle => "circle",
                ToolType::Rectangle => "rect",
                ToolType::Text => "Aa",
            };

            // Draw all entries
            self.canvas
                .actions
                .iter()
                .filter(|entry| entry.shown)
                .for_each(|entry| self.draw_action(entry));

            // Draw the tool icon
            draw_text(
                tool_icon,
                20.,
                screen_height() - 20.,
                30.,
                self.selected_colour.into(),
            );

            if is_quit_requested() || is_key_down(KeyCode::Escape) {
                self.show_exit_dialog = true;
            }

            if self.show_exit_dialog {
                self.draw_exit_dialog();
            }

            if self.user_decided_to_exit {
                self.tcp_packet_sender.send(TcpPacket::Disconnect).unwrap();
            }

            while let Ok(command) = self.canvas_receiver.try_recv() {
                self.process_command(command);
            }

            next_frame().await;
        }
    }

    fn draw_exit_dialog(&mut self) {
        let dialog_size = vec2(200., 70.);
        let screen_size = vec2(screen_width(), screen_height());
        let dialog_position = screen_size / 2. - dialog_size / 2.;
        Window::new(hash!(), dialog_position, dialog_size).ui(&mut root_ui(), |ui| {
            ui.label(None, "Do you really want to quit?");
            ui.separator();
            ui.same_line(60.);
            if ui.button(None, "Yes") {
                self.user_decided_to_exit = true;
            }
            ui.same_line(120.);
            if ui.button(None, "No") {
                self.show_exit_dialog = false;
            }
        });
    }
}

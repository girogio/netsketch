use super::commands::CanvasElement;

use bincode::{Decode, Encode};
use macroquad::{
    shapes::{draw_circle, draw_line, draw_rectangle},
    text::draw_text,
};

#[derive(Debug, Encode, Decode, Clone)]
pub struct CanvasAction {
    pub element: CanvasElement,
    pub user: String,
}

#[derive(Debug, Encode, Decode, Clone)]
pub struct CanvasEntry {
    pub id: usize,
    pub action: CanvasAction,
}

#[derive(Debug, Encode, Decode)]
pub struct Canvas {
    pub actions: Vec<CanvasEntry>,
    pub current_action_id: usize,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            current_action_id: 0,
        }
    }

    /// This function should only be called on the canvas by the server when a client draws
    /// something on the canvas.
    pub fn add_action(&mut self, user: String, element: &CanvasElement) -> CanvasEntry {
        let entry = CanvasEntry {
            id: self.current_action_id,
            action: CanvasAction {
                element: element.clone(),
                user,
            },
        };
        self.actions.push(entry.clone());
        self.current_action_id += 1;
        entry
    }

    /// This function should only be called in the same thread where the canvas
    /// provided by [`macroquad`] is being drawn.
    pub fn draw_action(&self, entry: &CanvasEntry) {
        match &entry.action.element {
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

    pub fn draw(&self) {
        self.actions
            .iter()
            .for_each(|action| self.draw_action(action));
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

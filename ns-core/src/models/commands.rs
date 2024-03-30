use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone)]
pub enum CanvasCommand {
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
    },
}

impl std::fmt::Display for CanvasCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CanvasCommand::Line { x1, y1, x2, y2 } => {
                write!(f, "Line: ({}, {}) -> ({}, {})", x1, y1, x2, y2)
            }
            CanvasCommand::Circle { x, y, radius } => {
                write!(f, "Circle: ({}, {}) - radius: {}", x, y, radius)
            }
            CanvasCommand::Rect {
                x,
                y,
                width,
                height,
            } => {
                write!(
                    f,
                    "Rect: ({}, {}) - width: {}, height: {}",
                    x, y, width, height
                )
            }
            CanvasCommand::Text { x, y, text } => {
                write!(f, "Text: ({}, {}) - {}", x, y, text)
            }
        }
    }
}

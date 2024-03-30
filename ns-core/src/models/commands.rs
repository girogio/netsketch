use bincode::{Decode, Encode};

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub enum CanvasCommand {
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Circle {
        x: f64,
        y: f64,
        radius: f64,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Text {
        x: f64,
        y: f64,
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

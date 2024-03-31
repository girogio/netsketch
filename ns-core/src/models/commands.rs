use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone)]
pub enum CanvasElement {
    Line {
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
        colour: [u8; 4],
    },
    Circle {
        x: u16,
        y: u16,
        radius: u16,
        colour: [u8; 4],
    },
    Rect {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        colour: [u8; 4],
    },
    Text {
        x: u16,
        y: u16,
        text: String,
        colour: [u8; 4],
    },
}

impl std::fmt::Display for CanvasElement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CanvasElement::Line {
                x1,
                y1,
                x2,
                y2,
                colour,
            } => {
                write!(
                    f,
                    "Line: ({}, {}) -> ({}, {}) in {:?}",
                    x1, y1, x2, y2, colour,
                )
            }
            CanvasElement::Circle {
                x,
                y,
                radius,
                colour,
            } => {
                write!(
                    f,
                    "Circle: ({}, {}) in {:?} - radius: {}",
                    x, y, colour, radius
                )
            }
            CanvasElement::Rect {
                x,
                y,
                width,
                height,
                colour,
            } => {
                write!(
                    f,
                    "Rect: ({}, {}) - width: {}, height: {} in {:?}",
                    x, y, width, height, colour,
                )
            }
            CanvasElement::Text { x, y, text, colour } => {
                write!(f, "Text: ({}, {}) - {} in {:?}", x, y, text, colour)
            }
        }
    }
}

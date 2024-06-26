use bincode::{Decode, Encode};

/// The different types of elements that can be drawn on the canvas.
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

#[derive(Debug, Encode, Decode, Clone)]
pub struct CanvasEntry {
    pub id: usize,
    pub shown: bool,
    pub element: CanvasElement,
    pub author: String,
}

#[derive(Clone)]
pub struct Canvas {
    pub entries: Vec<CanvasEntry>,
    pub current_action_id: usize,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_action_id: 0,
        }
    }

    pub fn add_action(&mut self, user: String, element: &CanvasElement) -> CanvasEntry {
        let entry = CanvasEntry {
            id: self.current_action_id,
            shown: true,
            element: element.clone(),
            author: user.clone(),
        };
        self.entries.push(entry.clone());
        self.current_action_id += 1;
        entry
    }

    pub fn get_entry(&self, id: usize) -> Option<&CanvasEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn update_entry(&mut self, id: usize, element: &CanvasElement) -> Option<CanvasEntry> {
        let index = self.entries.iter().position(|x| x.id == id);

        if let Some(index) = index {
            let entry = CanvasEntry {
                id,
                element: element.clone(),
                shown: self.entries[index].shown,
                author: self.entries[index].author.clone(),
            };
            self.entries[index] = entry.clone();
            Some(entry)
        } else {
            None
        }
    }

    pub fn delete_entry(&mut self, id: usize) {
        self.entries.retain(|entry| entry.id != id);
    }

    pub fn overwrite_entry(&mut self, id: usize, new_entry: CanvasEntry) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            *entry = new_entry;
        }
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CanvasEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] by [{}], {:?}", self.id, self.author, self.element)
    }
}

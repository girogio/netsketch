use ns_core::models::canvas::CanvasEntry;

#[derive(Clone)]
pub enum Action {
    Delete(CanvasEntry),
    Draw(usize),
    Update(CanvasEntry),
    Clear(ns_core::models::canvas::Canvas),
}

#[derive(Clone)]
pub struct UserData {
    pub name: String,
    pub action_history: Vec<Action>,
}

impl UserData {
    pub fn new(name: &str) -> Self {
        UserData {
            name: name.to_string(),
            action_history: vec![],
        }
    }
}

use std::time::Instant;

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
    pub username: String,
    pub action_history: Vec<Action>,
    pub last_login: Option<Instant>,
}

impl UserData {
    pub fn new(username: &str) -> Self {
        UserData {
            username: username.to_string(),
            action_history: vec![],
            last_login: None,
        }
    }
}

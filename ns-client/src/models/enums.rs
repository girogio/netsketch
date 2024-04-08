#[derive(Debug, Copy, Clone)]
pub enum ToolType {
    Line,
    Circle,
    Rectangle,
    Text,
}

#[derive(Debug, Clone)]
pub enum Ownership {
    Mine,
    All,
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub tool_type: Option<ToolType>,
    pub ownership: Ownership,
}

// SDL-like bindings skeleton for Wheel stdlib. This is a placeholder demonstrating API shape.

pub struct Window {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self { title: title.to_string(), width, height }
    }
    pub fn show(&self) {
        // Placeholder: real implementation would call system-specific graphics APIs
        println!("[sdl] show window: {} ({}x{})", self.title, self.width, self.height);
    }
}

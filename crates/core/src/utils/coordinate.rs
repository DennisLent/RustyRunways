use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

impl Coordinate {
    pub fn new(x: f32, y: f32) -> Self {
        Coordinate { x, y }
    }

    pub fn update(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }
}

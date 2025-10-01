use serde::{Deserialize, Serialize};

/// 2D world-space coordinate (x, y) used for airports and airplanes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

impl Coordinate {
    /// Create a new coordinate.
    ///
    /// Parameters
    /// - `x`: X position.
    /// - `y`: Y position.
    ///
    /// Returns
    /// - `Coordinate`: New coordinate.
    pub fn new(x: f32, y: f32) -> Self {
        Coordinate { x, y }
    }

    /// Translate this coordinate by (dx, dy) in place.
    ///
    /// Parameters
    /// - `dx`: Delta X.
    /// - `dy`: Delta Y.
    pub fn update(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }
}

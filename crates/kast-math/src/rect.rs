use crate::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }

    pub fn from_components(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }

    pub fn max(&self) -> Vec2 {
        Vec2::new(self.position.x + self.size.x, self.position.y + self.size.y)
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        let self_max = self.max();
        let other_max = other.max();

        self.position.x < other_max.x
            && self_max.x > other.position.x
            && self.position.y < other_max.y
            && self_max.y > other.position.y
    }
}

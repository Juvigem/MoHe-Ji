use super::vec2::Vec2;

#[derive(Debug, Clone, Default)]
pub struct PPWPolygon {
    pub polygon: Vec<Vec2>,
    pub segments: Vec<Vec<Vec2>>,
}

impl PPWPolygon {
    pub fn empty() -> Self {
        Self {
            polygon: Vec::new(),
            segments: Vec::new(),
        }
    }
}

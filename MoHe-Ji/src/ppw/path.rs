use super::vec2::Vec2;

#[derive(Debug, Clone)]
pub struct PPWPath {
    pub is_closed: bool,
    pub control_points: Vec<Vec2>,
    pub weights: Vec<f32>,
    pub phis: Vec<f32>,
    pub psis: Vec<f32>,
    pub fill_enabled: bool,
    pub fill_color: [u8; 4],
    pub stroke_width: f32,
    pub stroke_color: [u8; 4],
}

impl PPWPath {
    pub fn validate(&self) -> bool {
        let control_points_len = self.control_points.len();
        let weights_len = self.weights.len();
        let phis_len = self.phis.len();
        let psis_len = self.psis.len();

        if self.is_closed {
            control_points_len >= 3
                && control_points_len == weights_len
                && control_points_len == phis_len
                && control_points_len == psis_len
        } else {
            control_points_len >= 2
                && control_points_len == weights_len
                && control_points_len == phis_len + 1
                && control_points_len == psis_len + 1
        }
    }
}

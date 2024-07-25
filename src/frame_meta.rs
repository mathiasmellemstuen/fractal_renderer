use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FrameMeta {
    pub max_iterations : usize, 
    pub x_pos : f64, 
    pub y_pos : f64, 
    pub radius : f64,
    pub color_gradient_shift : f64
}

impl PartialEq for FrameMeta {
    fn eq(&self, other: &FrameMeta) -> bool {

        // We deliberatively do not want to check for the color gradient values when we check for
        // equality.
        self.max_iterations == other.max_iterations && self.x_pos == other.x_pos && self.y_pos == other.y_pos && self.radius == other.radius
    }
}

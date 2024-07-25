use crate::interpolation::*; 
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transition {
    pub from_frame : usize, 
    pub to_frame : usize,
    pub interpolation_type : InterpolationType,
    pub steps : usize
}

use serde::Deserialize;
use std::{f64::consts::PI};
use super::frame_meta::*; 

#[derive(Debug, Deserialize)]
pub enum InterpolationType {
    Linear,
    InOutQuart,
    InOutCubic,
    InSine,
    InOutSine,
    OutSine
}

impl InterpolationType {
    fn interpolate(&self, time : f64) -> f64 {
        match self {
            InterpolationType::Linear => time,
            InterpolationType::InOutQuart => ease_in_out_quart(time),
            InterpolationType::InOutCubic => ease_in_out_cubic(time),
            InterpolationType::InSine => ease_in_sine(time),
            InterpolationType::InOutSine => ease_in_out_sine(time),
            InterpolationType::OutSine => ease_out_sine(time)
        }
    }
}

fn ease_in_out_sine(time : f64) -> f64 {
    -((PI * time).cos() - 1.0) / 2.0
}
fn ease_in_out_quart(time : f64) -> f64{
    if time < 0.5 {8.0 * time.powi(4)} else {1.0 - (-2.0 * time + 2.0).powi(4) / 2.0}
}

fn ease_in_out_cubic(time : f64) -> f64 {
    if time < 0.5 {4.0 * time.powi(3)} else {1.0 - (-2.0 * time + 2.0).powi(3) / 2.0}
}

fn ease_in_sine(time : f64) -> f64 {
    1.0 - (time * PI / 2.0).cos()
}
fn ease_out_sine(time : f64) -> f64 {
    0.0 - ((time * PI / 2.0).cos()+ PI * 0.5)
}

fn lerp(a : f64, b : f64, t : f64) -> f64 {
    (a  * (1.0 - t)) + (b * t)
}

pub fn create_frame_from_lerp(from : &FrameMeta, to : &FrameMeta, time : f64) -> FrameMeta {

    let mi : f64 = lerp(from.max_iterations as f64, to.max_iterations as f64, time); 
    let x_pos : f64 = lerp(from.x_pos, to.x_pos, time); 
    let y_pos : f64 = lerp(from.y_pos, to.y_pos, time); 
    let radius : f64 = lerp(from.radius, to.radius, time); 
    let color_gradient_shift : f64 = lerp(from.color_gradient_shift, to.color_gradient_shift, time); 

    FrameMeta {
            max_iterations : mi.round() as usize,
            x_pos,
            y_pos,
            radius,
            color_gradient_shift
    }
}

pub fn interpolate_frames(frame_1 : &FrameMeta, frame_2 : &FrameMeta, steps : usize, mode : &InterpolationType) -> Vec<FrameMeta> {
    
    let mut all_steps : Vec<f64> = Vec::new(); 

    for step in 0 .. steps {
        all_steps.push((step as f64) * (1.0 / (steps as f64))); 
    }

    let mut all_frames : Vec<FrameMeta> = Vec::new();
    
    for step in 0 .. steps {
        let t : f64 = mode.interpolate(all_steps[step]); 
        all_frames.push(create_frame_from_lerp(&frame_1, &frame_2, t));
    }
    all_frames
}

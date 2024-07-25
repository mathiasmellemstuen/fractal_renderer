use crate::frame_meta::*;
use crate::transition::*; 
use crate::interpolation::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VideoFrames {
    frames: Vec<FrameMeta>,
    transitions : Vec<Transition>
}

impl VideoFrames {
    pub fn construct_all_frames(&self) -> Vec<FrameMeta> {
        let mut all_frames : Vec<FrameMeta> = Vec::new(); 

        for transition in &self.transitions {
            all_frames.extend(interpolate_frames(&self.frames[transition.from_frame], &self.frames[transition.to_frame], transition.steps, &transition.interpolation_type));
        }

        all_frames
    }
}

use crate::frame_meta::FrameMeta;
use video_rs::encode::{Encoder, Settings};
use video_rs::time::Time;
use std::path::Path;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use crate::frame_renderer::*;
use crate::fractal_function_type::FractalFunctionType;

pub fn create_mp4(x_size : usize, y_size : usize, frames_per_second : usize, frames : &Vec<FrameMeta>, color_gradient : &colorgrad::Gradient, fractal_f : FractalFunctionType) {
    let settings = Settings::preset_h264_yuv420p(x_size, y_size, false);
    let mut encoder = Encoder::new(Path::new("mandelbrot.mp4"), settings).expect("Failed to create encoder");

    let duration: Time = Time::from_nth_of_a_second(frames_per_second);
    let mut position = Time::zero();
    
    let progress_bar = ProgressBar::new(frames.len() as u64); 
    progress_bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, FPS {per_sec}, ETA {eta})",
        )
        .unwrap(),
    );

    for f in frames.iter() {
        
        let frame = create_frame_image_par(f.max_iterations, x_size, y_size, f.x_pos, f.y_pos, f.radius, &color_gradient, f.color_gradient_shift, fractal_f);

        encoder.encode(&frame, position).expect("Failed to encode video!");

        position = position.aligned_with(duration).add(); 

        progress_bar.inc(1); 
    }

    encoder.finish().expect("Failed to finish encoder");
}

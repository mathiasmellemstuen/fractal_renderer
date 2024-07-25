use rayon::prelude::*;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use ndarray::Array3;
use video_rs::encode::{Encoder, Settings};
use video_rs::time::Time;
use std::path::Path;
use indicatif::ProgressBar;
use serde::Deserialize;
use toml;
use std::fs;
use std::process::exit;
use std::env; 

#[derive(Debug, Deserialize)]
struct VideoFrames {
    frames: Vec<FrameMeta>,
    transitions : Vec<Transition>
}

impl VideoFrames {
    fn construct_all_frames(&self) -> Vec<FrameMeta> {
        let mut all_frames : Vec<FrameMeta> = Vec::new(); 

        for transition in &self.transitions {
            all_frames.extend(interpolate_frames(&self.frames[transition.from_frame], &self.frames[transition.to_frame], transition.steps, &transition.interpolation_type));
        }

        all_frames
    }
}

#[derive(Debug, Deserialize)]
struct Transition {
    from_frame : usize, 
    to_frame : usize,
    interpolation_type : InterpolationType,
    steps : usize
}
#[derive(Debug, Deserialize)]
struct FrameMeta {
    max_iterations : usize, 
    x_pos : f64, 
    y_pos : f64, 
    radius : f64,
    color_gradient_shift : f64
}

impl PartialEq for FrameMeta {
    fn eq(&self, other: &FrameMeta) -> bool {

        // We deliberatively do not want to check for the color gradient values when we check for
        // equality.
        self.max_iterations == other.max_iterations && self.x_pos == other.x_pos && self.y_pos == other.y_pos && self.radius == other.radius
    }
}

fn lerp(a : f64, b : f64, t : f64) -> f64 {
    (a  * (1.0 - t)) + (b * t)
}

fn ease_in_out_quart(time : f64) -> f64{
    if time < 0.5 {8.0 * time.powi(4)} else {1.0 - (-2.0 * time + 2.0).powi(4) / 2.0}
}

fn ease_in_out_cubic(time : f64) -> f64 {
    if time < 0.5 {4.0 * time.powi(3)} else {1.0 - (-2.0 * time + 2.0).powi(3) / 2.0}
}

fn ease_in_sine(time : f64) -> f64 {
    1.0 - (time * PI as f64 / 2.0).cos()
}

fn ease_out_sine(time : f64) -> f64 {
    (time * PI as f64 / 2.0).sin()
}

fn create_frame_from_lerp(from : &FrameMeta, to : &FrameMeta, time : f64) -> FrameMeta {

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

#[derive(Debug, Deserialize)]
enum InterpolationType {
    Linear,
    InOutQuart,
    InOutCubic,
    InSine,
    OutSine
}

fn interpolate_frames(frame_1 : &FrameMeta, frame_2 : &FrameMeta, steps : usize, mode : &InterpolationType) -> Vec<FrameMeta> {
    

    let mut all_steps : Vec<f64> = Vec::new(); 

    for step in 0 .. steps {
        all_steps.push((step as f64) * (1.0 / (steps as f64))); 
    }

    let mut all_frames : Vec<FrameMeta> = Vec::new();
    
    for step in 0 .. steps {
        
        let t : f64 = match mode {
            InterpolationType::Linear => all_steps[step],
            InterpolationType::InOutQuart => ease_in_out_quart(all_steps[step]),
            InterpolationType::InOutCubic => ease_in_out_cubic(all_steps[step]),
            InterpolationType::InSine => ease_in_sine(all_steps[step]),
            InterpolationType::OutSine => ease_out_sine(all_steps[step])
        };

        all_frames.push(create_frame_from_lerp(&frame_1, &frame_2, t));
    }
    all_frames
}


fn create_single_image_snapshot(max_iterations : usize, x_size : usize, y_size : usize, x_pos : f64, y_pos : f64, radius : f64, color_gradient : &colorgrad::Gradient, color_gradient_shift : f64) {

    let buffer = create_mandelbrot_buffer_image(max_iterations, x_size, y_size, x_pos, y_pos, radius); 
    let mut image_buffer = image::ImageBuffer::new(x_size as u32, y_size as u32);

    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {

            let mut g_value = (buffer.lock().unwrap()[x as usize][y as usize] as f64) / (max_iterations as f64) + color_gradient_shift;

            g_value = g_value - g_value.floor(); 

            let color = color_gradient.at(g_value).to_rgba8();
            *pixel = image::Rgb([color[0], color[1], color[2]]);
        }

        image_buffer.save(format!("mandelbrot_snapshot.png")).unwrap();
}
fn main() {
    
    let x_size : usize = 3200; 
    let y_size : usize = 1800; 

    let color_gradient = colorgrad::sinebow(); 

    let args: Vec<String> = env::args().collect();
    
    // In this case, we will create a single frame by values inserted in the command line
    if args.len() > 1 {
        
        let max_iterations : usize = args[1].parse().unwrap();
        let x_pos : f64 = args[2].parse().unwrap(); 
        let y_pos : f64 = args[3].parse().unwrap(); 
        let radius : f64 = args[4].parse().unwrap(); 
        let color_gradient_shift : f64 = args[5].parse().unwrap(); 

        create_single_image_snapshot(max_iterations, x_size, y_size, x_pos, y_pos, radius, &color_gradient, color_gradient_shift); 

        return; 
    }

    // Reading and parsing frames from the toml file
    let frames_file = "frames.toml"; 
    let frames_file_content = match fs::read_to_string(frames_file) {
        Ok(c) => c, 
        Err(_) => {
            eprintln!("Could not read the file!"); 
            exit(1); 
        }
    };
    let video_frames : VideoFrames = match toml::from_str(&frames_file_content) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Unable to parse file content!"); 
            exit(1);
        }
    };
    let frames = video_frames.construct_all_frames(); 


    let settings = Settings::preset_h264_yuv420p(x_size, y_size, false);
    let mut encoder = Encoder::new(Path::new("mandelbrot.mp4"), settings).expect("Failed to create encoder");

    let duration: Time = Time::from_nth_of_a_second(60);
    let mut position = Time::zero();
    
    let progress_bar = ProgressBar::new(frames.len() as u64); 
    
    let mut current_frame_meta : &FrameMeta = &frames[0]; 
    let mut buffer = create_mandelbrot_buffer_image(frames[0].max_iterations, x_size, y_size, frames[0].x_pos, frames[0].y_pos, frames[0].radius); 

    for f in frames.iter() {
        
        if f != current_frame_meta {
            buffer = create_mandelbrot_buffer_image(f.max_iterations, x_size, y_size, f.x_pos, f.y_pos, f.radius);
            current_frame_meta = f;
        }

        let frame = create_frame(x_size, y_size, f.max_iterations, &buffer, &color_gradient, f.color_gradient_shift); 
        
        encoder.encode(&frame, position).expect("Failed to encode video!");

        position = position.aligned_with(duration).add(); 

        progress_bar.inc(1); 
    }

    encoder.finish().expect("Failed to finish encoder");
}

fn create_mandelbrot_buffer_image(max_iterations : usize, x_size : usize, y_size : usize, x_pos : f64, y_pos : f64, radius : f64) -> Arc<Mutex<Vec<Vec<usize>>>> {

    let buffer = Arc::new(Mutex::new(vec![vec![0 as usize; y_size as usize]; x_size as usize]));

    let aspect : f64 = x_size as f64 / y_size as f64; 
    let x_radius = radius; 
    let y_radius = radius / aspect; 

    (0 .. x_size).into_par_iter().for_each(|x| {
        (0 .. y_size).into_par_iter().for_each(|y| {
        
            
            let x_0 : f64 = (x_pos - x_radius) + (x as f64 / x_size as f64) * ((x_pos + x_radius) - (x_pos - x_radius));
            let y_0 : f64 = (y_pos - y_radius) + (y as f64 / y_size as f64) * ((y_pos + y_radius) - (y_pos - y_radius));
            
            let mut current_x : f64 = 0.0; 
            let mut current_y : f64 = 0.0; 
            let mut iteration : usize = 0; 

            while current_x * current_x + current_y * current_y <= 4.0 && iteration < max_iterations {
                let x_temp = current_x * current_x - current_y * current_y + x_0; 

                current_y = 2.0 * current_x * current_y + y_0; 
                current_x = x_temp; 

                iteration += 1; 
            }

            buffer.lock().unwrap()[x][y] = iteration; 
        });
    });

    buffer
}

fn create_frame(x_size : usize, y_size : usize, max_iterations : usize, buffer : &Arc<Mutex<Vec<Vec<usize>>>>, color_gradient : &colorgrad::Gradient, color_gradient_shift : f64) -> Array3<u8> {
    
    let mut arr = Array3::<u8>::zeros((y_size, x_size, 3));

    for x in 0 .. x_size {
        for y in 0 .. y_size {
            let x : usize = x; 
            let y : usize = y;

            let mut g_value = (buffer.lock().unwrap()[x as usize][y as usize] as f64) / (max_iterations as f64) + color_gradient_shift;

            // We need to do this because we are shifting the value in the previous line. This is to
            // keep the final value between 0 - 1. 
            g_value = g_value - g_value.floor(); 

            let color = color_gradient.at(g_value).to_rgba8();
            arr[[y, x, 0]] = color[0];
            arr[[y, x, 1]] = color[1];
            arr[[y, x, 2]] = color[2];
        }
    }

    arr
}

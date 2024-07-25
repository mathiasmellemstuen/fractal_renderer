use toml;
use std::fs;
use std::process::exit;
use std::env; 
use serde::Deserialize;

use fractal_renderer::video_frames::*;
use fractal_renderer::snapshot::*;
use fractal_renderer::mp4::*;

fn str_to_colorgrad_gradient(in_str : &str) -> colorgrad::Gradient {
    match in_str {
        "sinebow" => colorgrad::sinebow(),
        "cubehelix_default" => colorgrad::cubehelix_default(), 
        "turbo" => colorgrad::turbo(),
        "spectral" => colorgrad::spectral(),
        "viridis" => colorgrad::viridis(),
        "magma" => colorgrad::magma(),
        "rainbow" => colorgrad::rainbow(),
        _ => {
            eprintln!("Color gradient string from properties file is unknown!"); 
            exit(1);
        }
    }
}
fn main() {

    let args: Vec<String> = env::args().collect();
    
    // In this case, we will create a single frame by values inserted in the command line
    if args.len() > 1 {
        let x_size : usize = args[1].parse().unwrap(); 
        let y_size : usize = args[2].parse().unwrap(); 
        let max_iterations : usize = args[3].parse().unwrap();
        let x_pos : f64 = args[4].parse().unwrap(); 
        let y_pos : f64 = args[5].parse().unwrap(); 
        let radius : f64 = args[6].parse().unwrap(); 
        let color_gradient_str : String = args[7].parse().unwrap(); 
        let color_gradient_shift : f64 = args[8].parse().unwrap(); 
        
        let color_gradient : colorgrad::Gradient = str_to_colorgrad_gradient(color_gradient_str.as_str()); 

        create_single_image_snapshot(max_iterations, x_size, y_size, x_pos, y_pos, radius, &color_gradient, color_gradient_shift); 
        return; 

    }
    // From here, we will create a video from the properties and frames toml files. 

    // Reading properties from toml file
    let properties_file = "properties.toml"; 
    let properties_file_content = match fs::read_to_string(properties_file) {
        Ok(c) => c, 
        Err(_) => {
            eprintln!("Could not read the file!"); 
            exit(1); 
        }
    };
    
    #[derive(Debug, Deserialize)]
    struct Properties {
        width : usize, 
        height : usize,
        frames_per_second : usize,
        colorgrad : String
    }

    let properties : Properties = match toml::from_str(&properties_file_content) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Unable to parse properties file content!"); 
            exit(1);
        }
    };

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
        Ok(c) => c,
        Err(_) => {
            eprintln!("Unable to parse file content!"); 
            exit(1);
        }
    };
    let frames = video_frames.construct_all_frames(); 
    
    let color_gradient : colorgrad::Gradient = str_to_colorgrad_gradient(properties.colorgrad.as_str()); 
    
    // Creating a mp4 video file from the frames and properties 
    create_mp4(properties.width, properties.height, properties.frames_per_second, &frames, &color_gradient);
}

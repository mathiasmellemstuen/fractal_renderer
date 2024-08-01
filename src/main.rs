use toml;
use std::fs;
use std::process::exit;
use serde::Deserialize;

use fractal_renderer::video_frames::*;
use fractal_renderer::mp4::*;
use fractal_renderer::fractals::*;

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

    // Reading properties from toml file
    let properties_file = "properties.toml"; 
    let properties_file_content = match fs::read_to_string(properties_file) {
        Ok(c) => c, 
        Err(e) => {
            eprintln!("Could not read the properties file!"); 
            println!("{}", e);
            exit(1); 
        }
    };
    
    #[derive(Debug, Deserialize)]
    struct Properties {
        fractal : String,
        width : usize, 
        height : usize,
        frames_per_second : usize,
        colorgrad : String
    }

    let properties : Properties = match toml::from_str(&properties_file_content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Unable to parse properties file content!"); 
            println!("{}", e);
            exit(1);
        }
    };

    // Reading and parsing frames from the toml file
    let frames_file = "frames.toml"; 
    let frames_file_content = match fs::read_to_string(frames_file) {
        Ok(c) => c, 
        Err(e) => {
            eprintln!("Could not read the file!"); 
            println!("{}", e);
            exit(1); 
        }
    };
    let video_frames : VideoFrames = match toml::from_str(&frames_file_content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Unable to parse file content!"); 
            println!("{}", e);
            exit(1);
        }
    };
    let frames = video_frames.construct_all_frames(); 
    
    let color_gradient : colorgrad::Gradient = str_to_colorgrad_gradient(properties.colorgrad.as_str()); 
    let fractal_f = str_to_fractal_fn(properties.fractal.as_str());
    
    // Creating a mp4 video file from the frames and properties 
    create_mp4(properties.width, properties.height, properties.frames_per_second, &frames, &color_gradient, fractal_f);
}

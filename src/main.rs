use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use ndarray::prelude::*;
use ndarray::Array3;
use ndarray::Array;
use video_rs::encode::{Encoder, Settings};
use video_rs::time::Time;
use std::path::Path;

struct FrameMeta {
    max_iterations : usize, 
    x_pos : f64, 
    y_pos : f64, 
    radius : f64,
    color_gradient_shift : f64
}

fn interpolate_linear(frame_1 : FrameMeta, frame_2 : FrameMeta, steps : usize) -> Vec<FrameMeta> {
    
    let max_iterations_step = (frame_1.max_iterations as f64 - frame_2.max_iterations as f64) / steps as f64; 
    let x_pos_step = (frame_1.x_pos - frame_2.x_pos) / steps as f64;
    let y_pos_step = (frame_1.y_pos - frame_2.y_pos) / steps as f64;
    let radius_step = (frame_1.radius - frame_2.radius) / steps as f64;
    let color_gradient_shift_step = (frame_1.color_gradient_shift - frame_2.color_gradient_shift) / steps as f64;

    let mut all_frames : Vec<FrameMeta> = Vec::new();
    
    for step in 0 .. steps {

        all_frames.push(FrameMeta{
            max_iterations : frame_1.max_iterations + (max_iterations_step * (step as f64)) as usize,
            x_pos : frame_1.x_pos + x_pos_step * (step as f64),
            y_pos : frame_1.y_pos + y_pos_step * (step as f64),
            radius : frame_1.radius + radius_step * (step as f64),
            color_gradient_shift : frame_1.color_gradient_shift + color_gradient_shift_step * (step as f64)
        });
    }
    all_frames
}

fn main() {

    let x_size : usize = 3200; 
    let y_size : usize = 1800; 
    let max_iterations : usize = 28;

    let x_pos = -0.16;
    let y_pos = 1.0405; 
    let radius = 0.01; 

    let color_gradient = colorgrad::sinebow(); 
    let color_gradient_shift = 0.4;

    let start_frame : FrameMeta = FrameMeta{
        max_iterations: 28,
        x_pos: -0.16,
        y_pos: 1.0405,
        radius: 0.01,
        color_gradient_shift: 0.0
    };

    let end_frame : FrameMeta = FrameMeta{
        max_iterations: 28,
        x_pos: -0.14,
        y_pos: 1.0405,
        radius: 0.01,
        color_gradient_shift: 0.7
    };

    let frames = interpolate_linear(start_frame, end_frame, 60);

    let settings = Settings::preset_h264_yuv420p(x_size, y_size, false);
    let mut encoder = Encoder::new(Path::new("mandelbrot.mp4"), settings).expect("Failed to create encoder");

    let duration: Time = Time::from_nth_of_a_second(60);
    let mut position = Time::zero();

    for (i, f) in frames.iter().enumerate() {
        let buffer = create_mandelbrot_buffer_image(f.max_iterations, x_size, y_size, f.x_pos, f.y_pos, f.radius); 
        let frame = create_frame(i, x_size, y_size, f.max_iterations, buffer, &color_gradient, f.color_gradient_shift); 

        encoder.encode(&frame, position).expect("Failed to encode video!"); 

        position = position.aligned_with(duration).add(); 
    }
}

fn create_mandelbrot_buffer_image(max_iterations : usize, x_size : usize, y_size : usize, x_pos : f64, y_pos : f64, radius : f64) -> Arc<Mutex<Vec<Vec<usize>>>> {

    let mut buffer = Arc::new(Mutex::new(vec![vec![0 as usize; y_size as usize]; x_size as usize]));

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
fn create_frame(frame_index : usize, x_size : usize, y_size : usize, max_iterations : usize, buffer : Arc<Mutex<Vec<Vec<usize>>>>, color_gradient : &colorgrad::Gradient, color_gradient_shift : f64) -> Array3<u8> {

    // let mut arr = Array3::zeros((x_size, y_size, 3 as usize));
    // let mut arr = Array::<u8, _>::zeros((x_size, y_size, 3).f());
    // let mut arr = Array::zeros((x_size, y_size, 3)); 

    return Array3::from_shape_fn((x_size, y_size, 3), |(x, y, c)| {
            let x : usize = x; 
            let y : usize = y;

            let mut g_value = (buffer.lock().unwrap()[x as usize][y as usize] as f64) / (max_iterations as f64) + color_gradient_shift;

            // We need to do this because we are shifting the value in the previous line. This is to
            // keep the final value between 0 - 1. 
            g_value = g_value - g_value.floor(); 

            let color = color_gradient.at(g_value).to_rgba8();
            // arr[x][y][0] = color[0];
            // arr[x][y][1] = color[1];
            // arr[x][y][2] = color[2];
            color

    });
    // for x in 0 .. x_size {
    //     for y in 0 .. y_size {
    //     }
    // }

    // arr

    // let mut image_buffer = image::ImageBuffer::new(x_size as u32, y_size as u32);

    // for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {

    //     let mut g_value = (buffer.lock().unwrap()[x as usize][y as usize] as f64) / (max_iterations as f64) + color_gradient_shift;

    //     // We need to do this because we are shifting the value in the previous line. This is to
    //     // keep the final value between 0 - 1. 
    //     g_value = g_value - g_value.floor(); 

    //     let color = color_gradient.at(g_value).to_rgba8();
    //     *pixel = image::Rgb([color[0], color[1], color[2]]);
    // }

    // image_buffer.save(format!("mandelbrot_{}.png", frame_index)).unwrap();
}

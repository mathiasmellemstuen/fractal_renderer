use std::process::exit;
use crate::fractal_function_type::FractalFunctionType; 

pub fn str_to_fractal_fn(in_str : &str) -> FractalFunctionType {
    match in_str {
        "mandelbrot" => mandelbrot,
        _ => {
            eprintln!("Fractal function string from properties file is unknown!"); 
            exit(1);
        }

    }
}

pub fn mandelbrot(max_iterations : usize, x : f64, y : f64, x_size : f64, y_size : f64, x_pos : f64, y_pos : f64, x_radius : f64, y_radius : f64) -> usize {

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

    iteration
}

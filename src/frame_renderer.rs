use ndarray::prelude::*;
use rayon::prelude::*;
use crate::fractal_function_type::FractalFunctionType;

pub fn create_frame_image_par(
    max_iterations : usize,
    x_size : usize,
    y_size : usize,
    x_pos : f64,
    y_pos : f64,
    radius : f64,
    color_gradient : &colorgrad::Gradient,
    color_gradient_shift : f64,
    fractal_f : FractalFunctionType
    ) -> Array3::<u8> {

    let aspect : f64 = x_size as f64 / y_size as f64; 
    let x_radius = radius; 
    let y_radius = radius / aspect; 

    let mut buffer = Array3::<u8>::zeros((y_size, x_size, 3));
    buffer.axis_iter_mut(Axis(0)).enumerate().par_bridge().for_each(|(y, mut row)| {
        row.axis_iter_mut(Axis(0)).enumerate().for_each(|(x, mut pixel)| {
            
            let iterations : usize = fractal_f(max_iterations, x as f64, y as f64, x_size as f64, y_size as f64, x_pos, y_pos, x_radius, y_radius);

            let mut value = (iterations as f64 / max_iterations as f64) + color_gradient_shift; 
            value = value - value.floor();
            
            let g_value = color_gradient.at(value).to_rgba8(); 

            pixel[0] = g_value[0];
            pixel[1] = g_value[1];
            pixel[2] = g_value[2];
        });
    });

    buffer
}

use ndarray::prelude::*;

pub fn create_mandelbrot_frame_image(max_iterations : usize, x_size : usize, y_size : usize, x_pos : f64, y_pos : f64, radius : f64, color_gradient : &colorgrad::Gradient, color_gradient_shift : f64) -> Array3::<u8> {

    let aspect : f64 = x_size as f64 / y_size as f64; 
    let x_radius = radius; 
    let y_radius = radius / aspect; 

    
    let mut buffer = Array3::<u8>::zeros((y_size, x_size, 3));

    buffer.axis_iter_mut(Axis(0)).enumerate().for_each(|(y, mut column)| {
        column.axis_iter_mut(Axis(0)).enumerate().for_each(|(x, mut channels)| {
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
            let mut value = (iteration as f64 / max_iterations as f64) + color_gradient_shift; 
            value = value - value.floor();
            
            let g_value = color_gradient.at(value).to_rgba8(); 

            channels[0] = g_value[0]; 
            channels[1] = g_value[1]; 
            channels[2] = g_value[2]; 
        });
    });

    buffer
}

use super::mandelbrot::*; 

pub fn create_single_image_snapshot(max_iterations : usize, x_size : usize, y_size : usize, x_pos : f64, y_pos : f64, radius : f64, color_gradient : &colorgrad::Gradient, color_gradient_shift : f64) {

    let buffer = create_mandelbrot_frame_image(max_iterations, x_size, y_size, x_pos, y_pos, radius, color_gradient, color_gradient_shift); 
    let mut image_buffer = image::ImageBuffer::new(x_size as u32, y_size as u32);

    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {

        *pixel = image::Rgb([buffer[[y as usize, x as usize, 0]], buffer[[y as usize, x as usize, 1]], buffer[[y as usize, x as usize, 2]]]);
    }

    image_buffer.save(format!("mandelbrot_snapshot.png")).unwrap();
}

struct FractalConfig {
    pos_x: f32,
    pos_y: f32,
    zoom: f32,
    max_iterations: u32,
	resolution_x : f32,
	resolution_y : f32,
}

@group(0)
@binding(0)
var<uniform> fractal_config: FractalConfig;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {

    // Define the four vertices of a rectangle over the whole screen
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // Bottom-left
        vec2<f32>( 1.0, -1.0), // Bottom-right
        vec2<f32>(-1.0,  1.0), // Top-left

        vec2<f32>(-1.0,  1.0), // Top-left
        vec2<f32>( 1.0, -1.0), // Bottom-right
        vec2<f32>( 1.0,  1.0)  // Top-right
    );

    // Fetch the correct position based on vertex index
    let pos = positions[in_vertex_index];

    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {

    let resolution : vec2<f32> = vec2<f32>(fractal_config.resolution_x, fractal_config.resolution_y);
    let uv : vec2<f32> = (frag_pos.xy / resolution) * 2.0 - vec2<f32>(1.0, 1.0);

	let center : vec2<f32> = vec2<f32>(fractal_config.pos_x, fractal_config.pos_y);

    // Map normalized coordinates to Mandelbrot space
    let c : vec2<f32> = uv * fractal_config.zoom + center;

    // Initialize z = 0 + 0i
    var z : vec2<f32> = vec2<f32>(0.0, 0.0);
    
	var i = 0; 
	var max_iterations = i32(fractal_config.max_iterations); 

    // Perform Mandelbrot iteration z = z^2 + c
    for (; i < max_iterations; i = i + 1) {
        // Calculate z^2 (real and imaginary parts)
        let z_real : f32 = z.x * z.x - z.y * z.y;
        let z_imag : f32 = 2.0 * z.x * z.y;
        
        // Update z to z^2 + c
        z = vec2<f32>(z_real, z_imag) + c;

        // Escape condition: if the magnitude of z exceeds 2.0
        if (dot(z, z) > 4.0) {
            break;
        }
    }

    let t : f32 = f32(i) / f32(max_iterations);
    return vec4<f32>(t * 0.9, t * 0.5, t * 0.3, 1.0);
}

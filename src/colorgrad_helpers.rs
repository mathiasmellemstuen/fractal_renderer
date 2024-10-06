use std::process::exit;

pub fn str_to_colorgrad_gradient(in_str : &str) -> colorgrad::Gradient {
    match in_str {
        "sinebow" => colorgrad::sinebow(),
        "cubehelix_default" => colorgrad::cubehelix_default(), 
        "turbo" => colorgrad::turbo(),
        "spectral" => colorgrad::spectral(),
        "viridis" => colorgrad::viridis(),
        "magma" => colorgrad::magma(),
        "rainbow" => colorgrad::rainbow().sharp(10, 0.4),
        _ => {
            eprintln!("Color gradient string from properties file is unknown!"); 
            exit(1);
        }
    }
}

use encase::ShaderType; 

#[derive(Debug, ShaderType)]
pub struct FractalConfig {
    pub cursor_pos_x : f32, 
    pub cursor_pos_y : f32,
    pub zoom : f32,
    pub max_iterations : u32,
    pub resolution_x : f32,
    pub resolution_y : f32
}

impl FractalConfig {
    pub fn as_wgsl_bytes(&self) -> encase::internal::Result<Vec<u8>> {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(self)?;
        Ok(buffer.into_inner())
    }
}

impl Default for FractalConfig {
    fn default() -> Self {
        FractalConfig {
            cursor_pos_x: 0.0,
            cursor_pos_y: 0.0,
            zoom: 2.3,
            max_iterations: 128,
            resolution_x : 1920.0,
            resolution_y : 1080.0,
        }
    }
}

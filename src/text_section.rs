use wgpu_text::glyph_brush::Section as TextSection;
use wgpu_text::glyph_brush::Text;

pub fn create_new_text_section(text : &str, position : (f32, f32)) -> TextSection {
    TextSection::default().add_text(Text::new(&text)
        .with_color([1.0, 1.0, 1.0, 1.0])
        .with_scale(24.0)
    ).with_screen_position(position)
}

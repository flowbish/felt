use image::Rgba;

#[derive(Debug, Copy, Clone)]
pub enum Color {
    White,
    LightGrey,
}

impl Color {
    pub fn rgba(self, alpha: f32) -> Rgba<u8> {
        return match self {
            Color::White => monochrome_pixel(245, alpha),
            Color::LightGrey => monochrome_pixel(180, alpha),
        };

        fn monochrome_pixel(v: u8, alpha: f32) -> Rgba<u8> {
            Rgba {
                data: [v, v, v, (255.0 * alpha) as u8],
            }
        }
    }
}

use std::path::Path;
use image::{DynamicImage, Pixel, Rgba, RgbaImage};

mod color;
mod lettering;

use crate::color::Color;
use crate::lettering::{Letter, Lettering};

const ROW_HEIGHT: f32 = 39.6 / 2.0;
const HEIGHT: f32 = ROW_HEIGHT * 5.0;

fn apply2_alpha<F>(bg: &mut Rgba<u8>, fg: &Rgba<u8>, mut f: F)
where
    F: FnMut(u8, u8, u8) -> u8,
{
    let fg_a = fg.data[3];
    for (bg, fg) in bg
        .channels_mut()
        .iter_mut()
        .take(3)
        .zip(fg.channels().iter())
    {
        *bg = f(*bg, *fg, fg_a);
    }
}

fn apply_overlay(bg: &mut Rgba<u8>, fg: &Rgba<u8>) {
    apply2_alpha(bg, fg, |bg, fg, fg_alpha| {
        let percent_fg = fg_alpha as f32 / 255.0;
        let percent_bg = 1.0 - percent_fg;

        ((bg as f32 * percent_bg) + (fg as f32 * percent_fg)) as u8
    });
}

fn new_image_tiled(image: &RgbaImage, width: u32, height: u32) -> RgbaImage {
    let repeat_width = (image.width() * 2) - 1;
    RgbaImage::from_fn(width, height, |x, y| {
        let x = x % repeat_width;
        let x = if x < image.width() {
            x
        } else {
            repeat_width - x
        };

        let y = y % image.height();
        image.get_pixel(x, y).clone()
    })
}

fn tile_image(image: &DynamicImage, tiles_x: u32, tiles_y: u32) -> RgbaImage {
    let image = image.to_rgba();
    let (width, height) = (image.width() / 2, image.height() / 2);
    let image = image::imageops::resize(&image, width, height, image::FilterType::Lanczos3);
    let image = new_image_tiled(&image, tiles_x * image.width(), tiles_y * image.height());
    image
}

pub struct LetteredBoard<'a> {
    board: &'a Board,
    lettering: Lettering<'a>,
}

impl<'a> LetteredBoard<'a> {
    fn render_glyph(
        image: &mut RgbaImage,
        height: u32,
        row: u32,
        width: f32,
        letter: Letter,
        color: Color,
    ) {
        letter.draw(|x, y, v| {
            let center_y = image.height() as f32 / 2.0;
            let height = height as f32 * HEIGHT;
            let y = ((center_y - (height / 2.0)) + ((row + 1) as f32 * HEIGHT)) as i32 + y;
            let x = ((image.width() as f32 / 2.0) - (width / 2.0)) as i32 + x;
            if x >= 0 && y >= 0 && (x as u32) < image.width() && (y as u32) < image.height() {
                let pixel = image.get_pixel_mut(x as u32, y as u32);
                apply_overlay(pixel, &color.rgba(v));
            }
        });
    }

    pub fn render(&self) -> RgbaImage {
        let mut image = self.board.background.image.clone();
        for (n, line) in self.lettering.lines().enumerate() {
            for (glyph, color) in line.letters() {
                LetteredBoard::render_glyph(
                    &mut image,
                    self.lettering.height(),
                    n as u32,
                    line.width(),
                    glyph,
                    color,
                );
            }
        }
        image
    }
}

pub struct Background {
    image: RgbaImage,
}

impl Background {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Background, image::ImageError> {
        let dyn_image = image::open(path)?;
        let image = tile_image(&dyn_image, 2, 2);
        Ok(Background { image })
    }
}

pub struct Font {
    inner: rusttype::Font<'static>,
}

impl Font {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Font, std::io::Error> {
        let font_bytes = std::fs::read(path)?;
        let collection = rusttype::FontCollection::from_bytes(font_bytes)?;
        Ok(Font { inner: collection.into_font()? })
    }
}

/// Represents the configuration for generating a board. This includes the background iamges and
/// fonts required to render the final image.
pub struct Board {
    background: Background,
    font: Font,
    shade: Font,
}

impl Board {
    pub fn new(background: Background, font: Font, shade: Font) -> Self {
        Board { background, font, shade }
    }

    pub fn write_phrase<'a>(&'a self, phrase: &str) -> LetteredBoard<'a> {
        let mut lettering = Lettering::new(&self.font.inner, &self.shade.inner);
        for letter in phrase.chars() {
            lettering.put_character(letter);
        }
        LetteredBoard {
            board: &self,
            lettering,
        }
    }
}

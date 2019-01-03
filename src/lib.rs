use image::{Pixel, Rgba, RgbaImage};
use std::path::Path;

mod color;
mod lettering;

use crate::color::Color;
use crate::lettering::{Letter, Lettering};

const ROWS_PER_LINE: u32 = 4;

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

pub struct LetteredBoard<'a> {
    board: &'a Board,
    lettering: Lettering<'a>,
}

impl<'a> LetteredBoard<'a> {
    fn render_glyph(
        image: &mut RgbaImage,
        row_height: f32,
        total_lines: u32,
        row: u32,
        width: f32,
        letter: Letter,
        color: Color,
    ) {
        letter.draw(|x, y, v| {
            let center_y = image.height() as f32 / 2.0;
            let height = row_height * ROWS_PER_LINE as f32 * total_lines as f32;
            let y = ((center_y - (height / 2.0)) + ((row + 1) as f32 * row_height * ROWS_PER_LINE as f32)) as i32 + y;
            let x = ((image.width() as f32 / 2.0) - (width / 2.0)) as i32 + x;
            if x >= 0 && y >= 0 && (x as u32) < image.width() && (y as u32) < image.height() {
                let pixel = image.get_pixel_mut(x as u32, y as u32);
                apply_overlay(pixel, &color.rgba(v));
            }
        });
    }

    fn composite_borders(&self, image: RgbaImage, border: &RgbaImage) -> RgbaImage {
        let border_width = border.width();
        let border_height = border.height();
        let background_width = image.width();
        let background_height = image.height();
        let left_side = border_width;
        let top_side = border_width;
        let right_side = left_side + background_width;
        let bottom_side = top_side + background_height;
        let width = right_side + border_width;
        let height = bottom_side + border_width;
        RgbaImage::from_fn(width, height, |x, y| match (x, y) {
            (x, y) if x < left_side && x < y && x < height - y => {
                *border.get_pixel(x, y % border_height)
            }
            (x, y) if y < left_side && y <= x && y < width - x => {
                *border.get_pixel(y, border_height - (x % border_height) - 1)
            }
            (x, y) if x >= right_side && x > y => {
                *border.get_pixel(x - right_side, y % border_height)
            }
            (x, y) if y >= border_width + background_height && y >= x => {
                *border.get_pixel(y - bottom_side, border_height - (x % border_height) - 1)
            }
            (x, y) => *image.get_pixel(x - top_side, y - left_side),
        })
    }

    pub fn render(&self) -> RgbaImage {
        let mut image = self.board.background.image.clone();
        for (n, line) in self.lettering.lines().enumerate() {
            for (glyph, color) in line.letters() {
                LetteredBoard::render_glyph(
                    &mut image,
                    self.board.background.row_height,
                    self.lettering.height(),
                    n as u32,
                    line.width(),
                    glyph,
                    color,
                );
            }
        }
        if let Some(ref border) = self.board.border {
            self.composite_borders(image, &border.image)
        } else {
            image
        }
    }
}

pub struct Background {
    image: RgbaImage,
    /// The height of each row.
    row_height: f32,
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

impl Background {
    pub fn open<P: AsRef<Path>>(path: P, rows: u32) -> Result<Background, image::ImageError> {
        let image = image::open(path)?.to_rgba();
        let (width, height) = (image.width() / 2, image.height() / 2);
        let row_height = height as f32 / rows as f32;
        let image = image::imageops::resize(&image, width, height, image::FilterType::Lanczos3);
        let image = new_image_tiled(&image, 2 * image.width(), 2 * image.height());
        Ok(Background { image, row_height })
    }
}

pub struct Border {
    image: RgbaImage,
}

impl Border {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Border, image::ImageError> {
        let image = image::open(path)?.to_rgba();
        Ok(Border { image })
    }
}

pub struct Font {
    inner: rusttype::Font<'static>,
}

impl Font {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Font, std::io::Error> {
        let font_bytes = std::fs::read(path)?;
        let collection = rusttype::FontCollection::from_bytes(font_bytes)?;
        Ok(Font {
            inner: collection.into_font()?,
        })
    }
}

/// Represents the configuration for generating a board. This includes the background iamges and
/// fonts required to render the final image.
pub struct Board {
    background: Background,
    border: Option<Border>,
    font: Font,
    shade: Font,
}

impl Board {
    pub fn new(background: Background, border: Option<Border>, font: Font, shade: Font) -> Self {
        Board {
            background,
            border,
            font,
            shade,
        }
    }

    pub fn write_phrase<'a>(&'a self, phrase: &str) -> LetteredBoard<'a> {
        let mut lettering = Lettering::new(self.background.row_height, &self.font.inner, &self.shade.inner);
        for letter in phrase.chars() {
            lettering.put_character(letter);
        }
        LetteredBoard {
            board: &self,
            lettering,
        }
    }
}

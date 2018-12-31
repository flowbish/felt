use crate::color::Color;
use crate::ROW_HEIGHT;
use rusttype::{point, Font, Glyph, GlyphId, PositionedGlyph, Scale};

const SCALE: f32 = ROW_HEIGHT * 5.8;

type Point = rusttype::Point<f32>;

pub struct Letter<'a, 'b> {
    glyph: &'b PositionedGlyph<'a>,
}

impl<'a, 'b: 'a> Letter<'a, 'b> {
    fn new(glyph: &'b PositionedGlyph) -> Self {
        Self { glyph }
    }

    pub fn draw<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, f32),
    {
        if let Some(bounding_box) = self.glyph.pixel_bounding_box() {
            self.glyph.draw(|x, y, v| {
                let (x, y) = (bounding_box.min.x + x as i32, bounding_box.min.y + y as i32);
                f(x, y, v);
            });
        }
    }
}

struct Cursor {
    position: Point,
    last: Option<GlyphId>,
}

impl Cursor {
    fn new() -> Cursor {
        Cursor {
            position: point(0.0, 0.0),
            last: None,
        }
    }
}

pub struct Line {
    glyphs: Vec<(PositionedGlyph<'static>, Color)>,
    cursor: Cursor,
}

impl Line {
    fn new() -> Line {
        Line {
            glyphs: Vec::new(),
            cursor: Cursor::new(),
        }
    }

    pub fn width(&self) -> f32 {
        self.glyphs.iter().fold(0.0, |width, (glyph, _color)| {
            glyph.pixel_bounding_box().map(|rect| {
                let max = rect.max.x as f32;
                if max > width { max } else { width }
            }).unwrap_or(width)
        })
    }

    fn advance_cursor(&mut self, font: &Font<'static>, id: GlyphId) {
        let width = font
            .glyph(id)
            .scaled(Scale::uniform(SCALE))
            .h_metrics()
            .advance_width;
        let padding = self
            .cursor
            .last
            .map(|last| font.pair_kerning(Scale::uniform(SCALE), last, id))
            .unwrap_or(0.0);

        self.cursor = Cursor {
            position: self.cursor.position + rusttype::vector(width + padding, 0.0),
            last: Some(id),
        }
    }

    fn put_glyph(&mut self, position: Point, glyph: Glyph<'static>, color: Color) {
        let scaled = glyph.scaled(Scale::uniform(SCALE));
        let positioned = scaled.positioned(position);
        self.glyphs.push((positioned, color));
    }

    fn put_letter(&mut self, font: &Font<'static>, shade: &Font<'static>, letter: char) {
        let glyph = font.glyph(letter);

        self.put_glyph(self.cursor.position, font.glyph(letter), Color::White);
        self.put_glyph(self.cursor.position, shade.glyph(letter), Color::LightGrey);

        self.advance_cursor(font, glyph.id());
    }

    pub fn letters<'b>(&'b self) -> impl Iterator<Item = (Letter<'b, 'b>, Color)> {
        self.glyphs.iter().map(|(g, c)| (Letter::new(g), *c))
    }
}

/// Represents the mutable lettering state.
pub struct Lettering<'a> {
    font: &'a Font<'static>,
    shade: &'a Font<'static>,
    lines: Vec<Line>,
}

impl<'a> Lettering<'a> {
    pub fn new(font: &'a Font<'static>, shade: &'a Font<'static>) -> Self {
        Lettering {
            font,
            shade,
            lines: vec![Line::new()],
        }
    }

    /// The height of this lettering in number of lines.
    pub fn height(&self) -> u32 {
        self.lines.len() as u32
    }

    fn put_letter(&mut self, letter: char) {
        if let Some(line) = self.lines.last_mut() {
            line.put_letter(&self.font, &self.shade, letter);
        }
    }

    fn put_new_line(&mut self) {
        self.lines.push(Line::new());
    }

    pub fn put_character(&mut self, c: char) {
        match c {
            '\n' => self.put_new_line(),
            letter => self.put_letter(letter),
        }
    }

    pub fn lines(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter()
    }
}

use felt::{Font, Background, Board, Border};
use image::RgbaImage;

fn default_board() -> Board {
    let background = Background::open("gray.jpg").unwrap();
    let border = Border::open("oak.jpg").unwrap();
    let font = Font::open("font.ttf").unwrap();
    let shade = Font::open("shade.ttf").unwrap();
    Board::new(background, Some(border), font, shade)
}

fn assert_images_equal(expected: &RgbaImage, image: &RgbaImage) {
    expected.enumerate_pixels().zip(image.pixels()).for_each(|((x, y, e), p)| {
        assert_eq!(e, p, "Pixels differ at ({}, {})", x, y);
    });
}

#[test]
fn empty_board() {
    let board = default_board();
    let image = board.write_phrase("").render();
    let expected = image::open("tests/empty.png").unwrap().to_rgba();
    assert_images_equal(&expected, &image);
}

#[test]
fn hello_world() {
    let board = default_board();
    let image = board.write_phrase("hello, world!").render();
    let expected = image::open("tests/hello_world.png").unwrap().to_rgba();
    assert_images_equal(&expected, &image);
}

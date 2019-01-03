use felt::{Font, Background, Board, Border};

// Create a board that loads in the background and font.
// Interact with the board to write a phrase and get a LetteredBoard.
// Use the LetteredBoard to render the letters into an image.

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let phrase = &args[1];

    let background = Background::open("gray.jpg", 16).expect("Unable to open background");
    let border = Border::open("oak.jpg").expect("Unable to open border");
    let font = Font::open("font.ttf").expect("Unable to open font");
    let shade = Font::open("shade.ttf").expect("Unable to open shade");
    let board = Board::new(background, Some(border), font, shade);

    let board = board.write_phrase(phrase);
    let image = board.render();
    image.save("output.png").expect("Error saving image");
}

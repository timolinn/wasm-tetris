mod utils;
pub mod tetrimino;

use wasm_bindgen::prelude::*;

pub use tetrimino::*;
use std::time::Duration;


// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
        console_error_panic_hook::set_once();

    Ok(())
}

enum State {
    Playing,
    Lost,
}

struct ActivePiece {
    piece: Tetrimino,
    x: isize,
    y: isize,
}

#[wasm_bindgen]
pub struct Game {
    width: usize,
    height: usize,
    board: Vec<Color>,

    // generator to generate next pieces randomly,
    // next_pieces will always contain three pieces according to the ui
    generator: SevenGenerator,
    next_pieces: Vec<PieceType>,

    // current playing piece and it's x, y coordinate
    // x, y coordinate is isize because the pieces can go out of bound
    active_piece: ActivePiece,

    // type of tetrimino currently being held, it can be empty
    // can_hold flag to indicate whether the player can hold the active piece
    // when a piece is held, it can be exchanged only on when the active piece change
    held_type: Option<PieceType>,
    can_hold: bool,

    state: State,

    // elapsed time since the last update
    elapsed: Duration,

    // fall rate defines how fast a piece fall
    // if time delta greater than this is elapsed, we move one piece down by one unit
    fall_rate: Duration,

    // indexes of lines which can be cleared/erased
    clearable_lines: Vec<u8>,

    // vector to hold pairs of x,y coordinates for current active piece's individual squares
    // have to do this way because there is no other good way to pass a vector
    // without incurring performance cost for serializing into js
    active_piece_coords: Vec<usize>,
}

#[wasm_bindgen]
impl Game {
    pub fn new(width: usize, height: usize) -> Game {
        let mut generator = SevenGenerator::new();
        let mut next_pieces = Vec::new();
        next_pieces.push(generator.next().unwrap());
        next_pieces.push(generator.next().unwrap());
        next_pieces.push(generator.next().unwrap());

        let active_piece = Self::initialize_tetrimino(generator.next().unwrap());

        Game {
            width,
            height,
            board: vec![Color::None; width * height],
            generator,
            next_pieces,
            active_piece,
            held_type: None,
            can_hold: true,
            state: State::Playing,
            elapsed: Duration::from_micros(0),
            fall_rate: Duration::from_millis(500), // TODO: this should update
            clearable_lines: Vec::new(),
            active_piece_coords: Vec::new(),
        }
    }

    pub fn board(&self) -> *const Color {
        self.board.as_ptr()
    }

    pub fn next_pieces(&self) -> *const PieceType {
        self.next_pieces.as_ptr()
    }

    pub fn active_piece(&self) -> *const u8 {
        unimplemented!()
    }

    pub fn update(&mut self, elapsed: u64) {
        let elapsed = Duration::from_micros(elapsed);
        self.elapsed += elapsed;

        if self.elapsed >= self.fall_rate {
            self.elapsed -= self.fall_rate;
            self.active_piece.y += 1;
        }
    }

    // apply displacements to move the next active tetrimino into the center of board
    fn initialize_tetrimino(ttype: PieceType) -> ActivePiece {
        let (x, y) = match ttype {
            PieceType::I => (3, -1),
            _ => (3, 0)
        };

        let current_tetrimino = ActivePiece {
            piece: Tetrimino::from(ttype),
            x,
            y,
        };

        current_tetrimino
    }
}

impl Game {
    fn get_index(&self, row: usize, col: usize) -> usize {
        (row * self.width) + col
    }
}


// Random generator to generate permutation of 7 pieces of tetriminos
// according to the guideline here:
// https://tetris.fandom.com/wiki/Random_Generator
struct SevenGenerator {
    shuffled: [PieceType; 7],
    index: usize,
}

impl SevenGenerator {
    fn new() -> SevenGenerator {
        use PieceType::*;

        let mut shuffled = [I, O, T, S, Z, J, L];
        utils::shuffle(&mut shuffled);

        SevenGenerator {
            shuffled,
            index: 0,
        }
    }
}

impl Iterator for SevenGenerator {
    type Item = PieceType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 7 {
            use PieceType::*;

            let mut shuffled = [I, O, T, S, Z, J, L];
            utils::shuffle(&mut shuffled);

            self.shuffled = shuffled;
            self.index = 0;
        }

        let result = self.shuffled[self.index];
        self.index += 1;
        Some(result)
    }
}
use serde::Serialize;
use crate::config::*;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum PieceKind {
    I, O, T, S, Z, J, L
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub struct Cell {
    pub kind: Option<PieceKind>,
    pub is_ghost: bool,
}

impl Cell {
    pub fn empty() -> Self {
        Self { kind: None, is_ghost: false }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub kind: PieceKind,
    pub rotation: u8, // 0-3
    pub x: i8,        // Board column (0-9), spawn at 3
    pub y: i8,        // Board row (0-19), spawn at 0 (top)
}

#[derive(Clone, Debug, Serialize)]
pub struct GameState {
    pub board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current_piece: Option<Piece>,
    pub next_piece: Piece,
    pub score: u32,
    pub level: u8,
    pub lines_cleared: u32,
    pub trash_streak: u8,
    pub lock_delay_ms: u32,
    pub lock_resets: u8,
    pub is_paused: bool,
    pub is_game_over: bool,
}

/// Piece shape definitions: [PieceKind][Rotation][Block] = (x, y)
/// Using simple 90° rotation (no wall kicks)
/// Origin is at piece's local (0,0) - top-left of bounding box
pub const PIECE_SHAPES: [[[(i8, i8); 4]; 4]; 7] = [
    // I
    [
        [(0, 0), (1, 0), (2, 0), (3, 0)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
        [(0, 0), (1, 0), (2, 0), (3, 0)],
        [(2, 0), (2, 1), (2, 2), (2, 3)],
    ],
    // O (all rotations identical)
    [
        [(0, 0), (1, 0), (0, 1), (1, 1)],
        [(0, 0), (1, 0), (0, 1), (1, 1)],
        [(0, 0), (1, 0), (0, 1), (1, 1)],
        [(0, 0), (1, 0), (0, 1), (1, 1)],
    ],
    // T
    [
        [(0, 0), (1, 0), (2, 0), (1, 1)],
        [(1, 0), (1, 1), (1, 2), (0, 1)],
        [(0, 1), (1, 1), (2, 1), (1, 0)],
        [(1, 0), (0, 1), (1, 1), (1, 2)],
    ],
    // S
    [
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
        [(1, 0), (2, 0), (0, 1), (1, 1)],
        [(1, 0), (1, 1), (2, 1), (2, 2)],
    ],
    // Z
    [
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
        [(0, 0), (1, 0), (1, 1), (2, 1)],
        [(2, 0), (1, 1), (2, 1), (1, 2)],
    ],
    // J
    [
        [(0, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (2, 0), (1, 1), (1, 2)],
        [(0, 0), (1, 0), (2, 0), (2, 1)],
        [(1, 0), (1, 1), (0, 2), (1, 2)],
    ],
    // L
    [
        [(2, 0), (0, 1), (1, 1), (2, 1)],
        [(1, 0), (1, 1), (1, 2), (2, 2)],
        [(0, 0), (1, 0), (2, 0), (0, 1)],
        [(0, 0), (1, 0), (0, 1), (0, 2)],
    ],
];

impl PieceKind {
    pub fn shapes(&self) -> &[[(i8, i8); 4]; 4] {
        &PIECE_SHAPES[*self as usize]
    }
}

/// 7-bag randomizer: shuffles all 7 pieces, yields them in order, then reshuffles
pub struct BagRandomizer {
    bag: Vec<PieceKind>,
    rng: StdRng,
}

impl BagRandomizer {
    pub fn new() -> Self {
        let mut bag: Vec<PieceKind> = vec![
            PieceKind::I, PieceKind::O, PieceKind::T,
            PieceKind::S, PieceKind::Z, PieceKind::J, PieceKind::L,
        ];
        let mut rng = StdRng::from_entropy();
        bag.shuffle(&mut rng);
        Self { bag, rng }
    }

    pub fn next(&mut self) -> PieceKind {
        if self.bag.is_empty() {
            self.refill();
        }
        self.bag.pop().unwrap()
    }

    fn refill(&mut self) {
        self.bag = vec![
            PieceKind::I, PieceKind::O, PieceKind::T,
            PieceKind::S, PieceKind::Z, PieceKind::J, PieceKind::L,
        ];
        self.bag.shuffle(&mut self.rng);
    }
}

impl Default for BagRandomizer {
    fn default() -> Self {
        Self::new()
    }
}
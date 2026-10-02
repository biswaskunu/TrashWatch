use serde::Serialize;
use crate::config::*;

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
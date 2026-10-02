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

impl GameState {
    pub fn new() -> Self {
        let mut bag = BagRandomizer::new();
        let first = bag.next();
        let second = bag.next();

        let mut state = Self {
            board: [[Cell::empty(); BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: Some(Piece {
                kind: first,
                rotation: 0,
                x: SPAWN_X,
                y: SPAWN_Y,
            }),
            next_piece: Piece {
                kind: second,
                rotation: 0,
                x: SPAWN_X,
                y: SPAWN_Y,
            },
            score: 0,
            level: 0,
            lines_cleared: 0,
            trash_streak: 0,
            lock_delay_ms: 0,
            lock_resets: 0,
            is_paused: false,
            is_game_over: false,
        };

        // Check for immediate game over on spawn
        if state.current_piece.as_ref().map_or(false, |p| state.collides(p, 0, 0, 0)) {
            state.is_game_over = true;
        }

        state
    }

    fn spawn_next(&mut self, bag: &mut BagRandomizer) {
        let next_kind = bag.next();
        self.current_piece = Some(Piece {
            kind: next_kind,
            rotation: 0,
            x: SPAWN_X,
            y: SPAWN_Y,
        });
        self.next_piece = Piece {
            kind: bag.next(),
            rotation: 0,
            x: SPAWN_X,
            y: SPAWN_Y,
        };

        // Game over if new piece collides immediately
        if self.current_piece.as_ref().map_or(false, |p| self.collides(p, 0, 0, 0)) {
            self.is_game_over = true;
        }
    }

    /// Check if piece collides at given offset
    fn collides(&self, piece: &Piece, dx: i8, dy: i8, drot: i8) -> bool {
        let rot = ((piece.rotation as i8 + drot) & 3) as usize;
        for (px, py) in self.kind_shapes(piece.kind)[rot] {
            let x = piece.x + px + dx;
            let y = piece.y + py + dy;
            if x < 0 || x >= BOARD_WIDTH as i8 || y >= BOARD_HEIGHT as i8 {
                return true;
            }
            if y >= 0 && self.board[y as usize][x as usize].kind.is_some() {
                return true;
            }
        }
        false
    }

    fn kind_shapes(&self, kind: PieceKind) -> &[[(i8, i8); 4]; 4] {
        &PIECE_SHAPES[kind as usize]
    }

    /// Try to move left, returns true if successful
    pub fn move_left(&mut self) -> bool {
        if let Some(piece) = self.current_piece {
            if !self.collides(&piece, -1, 0, 0) {
                self.current_piece.as_mut().unwrap().x -= 1;
                self.reset_lock_delay();
                return true;
            }
        }
        false
    }

    /// Try to move right, returns true if successful
    pub fn move_right(&mut self) -> bool {
        if let Some(piece) = self.current_piece {
            if !self.collides(&piece, 1, 0, 0) {
                self.current_piece.as_mut().unwrap().x += 1;
                self.reset_lock_delay();
                return true;
            }
        }
        false
    }

    /// Try to rotate clockwise, returns true if successful
    pub fn rotate_cw(&mut self) -> bool {
        if let Some(piece) = self.current_piece {
            if !self.collides(&piece, 0, 0, 1) {
                self.current_piece.as_mut().unwrap().rotation = (piece.rotation + 1) & 3;
                self.reset_lock_delay();
                return true;
            }
        }
        false
    }

    /// Try to rotate counter-clockwise, returns true if successful
    pub fn rotate_ccw(&mut self) -> bool {
        if let Some(piece) = self.current_piece {
            if !self.collides(&piece, 0, 0, -1) {
                self.current_piece.as_mut().unwrap().rotation = (piece.rotation + 3) & 3;
                self.reset_lock_delay();
                return true;
            }
        }
        false
    }

    /// Soft drop: move down one cell, returns true if moved
    pub fn soft_drop(&mut self) -> bool {
        if let Some(piece) = self.current_piece {
            if !self.collides(&piece, 0, 1, 0) {
                self.current_piece.as_mut().unwrap().y += 1;
                self.score += 1;
                self.lock_delay_ms = 0;
                return true;
            }
        }
        false
    }

    /// Hard drop: drop to bottom, lock immediately, returns points earned
    pub fn hard_drop(&mut self) -> u32 {
        let mut points = 0;
        while let Some(piece) = self.current_piece {
            if !self.collides(&piece, 0, 1, 0) {
                self.current_piece.as_mut().unwrap().y += 1;
                points += 2;
            } else {
                break;
            }
        }
        self.lock_piece();
        points
    }

    fn reset_lock_delay(&mut self) {
        self.lock_delay_ms = 0;
        self.lock_resets = self.lock_resets.saturating_add(1);
    }

    /// Lock current piece to board, clear lines, spawn next
    fn lock_piece(&mut self) {
        if let Some(piece) = self.current_piece.take() {
            // Write piece to board
            for (px, py) in self.kind_shapes(piece.kind)[piece.rotation as usize] {
                let x = piece.x + px;
                let y = piece.y + py;
                if x >= 0 && x < BOARD_WIDTH as i8 && y >= 0 && y < BOARD_HEIGHT as i8 {
                    self.board[y as usize][x as usize] = Cell { kind: Some(piece.kind), is_ghost: false };
                }
            }
            // Clear lines and update score
            let cleared = self.clear_lines();
            if cleared > 0 {
                self.add_score(cleared);
            }
            // Spawn next piece (will be called from tick with bag)
            // Note: spawn_next requires bag, so we set current_piece to None
            // and tick will call spawn_next when current_piece is None
        }
    }

    /// Try to lock piece if lock delay exceeded or max resets reached
    fn try_lock(&mut self) -> bool {
        if self.lock_delay_ms >= LOCK_DELAY_MS || self.lock_resets >= MAX_LOCK_RESETS {
            self.lock_piece();
            true
        } else {
            false
        }
    }

    /// Clear completed lines, return count
    fn clear_lines(&mut self) -> u8 {
        let mut cleared = 0;
        for y in (0..BOARD_HEIGHT).rev() {
            if self.board[y].iter().all(|c| c.kind.is_some()) {
                cleared += 1;
                // Shift down
                for yy in (1..=y).rev() {
                    self.board[yy] = self.board[yy - 1];
                }
                self.board[0] = [Cell::empty(); BOARD_WIDTH];
            }
        }
        cleared
    }

    /// Add score for cleared lines, update level and trash streak
    fn add_score(&mut self, lines: u8) {
        let base = match lines {
            1 => SCORE_SINGLE,
            2 => SCORE_DOUBLE,
            3 => SCORE_TRIPLE,
            4 => SCORE_QUAD,
            _ => 0,
        };
        self.score += base * (self.level as u32 + 1);
        self.lines_cleared += lines as u32;
        self.level = (self.lines_cleared / LINES_PER_LEVEL) as u8;
        self.trash_streak = self.trash_streak.saturating_add(1);
    }

    /// Main game tick: gravity + lock delay
    pub fn tick(&mut self, dt_ms: u32, bag: &mut BagRandomizer) {
        if self.is_paused || self.is_game_over {
            return;
        }

        // Spawn new piece if needed
        if self.current_piece.is_none() {
            self.spawn_next(bag);
            return;
        }

        // Gravity
        let frames_per_cell = GRAVITY_TABLE[self.level.min(29) as usize];
        let ms_per_cell = frames_per_cell * TICK_MS;

        // Simple gravity: accumulate time and move when threshold reached
        static mut GRAVITY_ACCUM: u32 = 0;
        unsafe {
            GRAVITY_ACCUM += dt_ms;
            while GRAVITY_ACCUM >= ms_per_cell {
                if let Some(piece) = self.current_piece {
                    if !self.collides(&piece, 0, 1, 0) {
                        self.current_piece.as_mut().unwrap().y += 1;
                        self.lock_delay_ms = 0;
                    } else {
                        self.lock_delay_ms += dt_ms;
                        if self.try_lock() {
                            GRAVITY_ACCUM = 0;
                            break;
                        }
                    }
                }
                GRAVITY_ACCUM -= ms_per_cell;
            }
        }
    }

    /// Get ghost piece position (where piece would land)
    pub fn ghost_piece(&self) -> Option<Piece> {
        let mut ghost = self.current_piece?;
        while !self.collides(&ghost, 0, 1, 0) {
            ghost.y += 1;
        }
        Some(ghost)
    }
}
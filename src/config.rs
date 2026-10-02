// Game Timing
pub const TICK_RATE_HZ: u64 = 60;
pub const TICK_MS: u32 = 1000 / TICK_RATE_HZ; // 16ms
pub const BROADCAST_RATE_HZ: u64 = 10;
pub const BROADCAST_INTERVAL_MS: u64 = 1000 / BROADCAST_RATE_HZ; // 100ms

// Lock Delay
pub const LOCK_DELAY_MS: u32 = 500;
pub const MAX_LOCK_RESETS: u8 = 15;

// Gravity (frames per cell at each level)
// Level 0: 48 frames (~800ms), Level 10: 6 frames (~100ms), Level 20+: 1 frame
pub const GRAVITY_TABLE: [u32; 30] = [
    48, 43, 38, 33, 28, 23, 18, 13, 8, 6,
    5, 5, 5, 4, 4, 4, 3, 3, 3, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 1,
];

// Scoring
pub const SCORE_SINGLE: u32 = 100;
pub const SCORE_DOUBLE: u32 = 300;
pub const SCORE_TRIPLE: u32 = 500;
pub const SCORE_QUAD: u32 = 800;
pub const LINES_PER_LEVEL: u32 = 10;

// Board
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const SPAWN_X: i8 = 3;
pub const SPAWN_Y: i8 = 0;

// Spectator
pub const REACTION_TTL_MS: u32 = 2000;
pub const REACTION_EMOJIS: &[&str] = &["🔥", "💀", "🚀", "✨"];
pub const MAX_CHAT_LENGTH: usize = 200;
pub const WS_CHANNEL_CAP: usize = 64;

// Terminal
pub const CELL_CHAR: &str = "██";
pub const GHOST_CHAR: &str = "░░";
pub const EMPTY_CHAR: &str = "  ";

// Colors (ANSI 256)
pub mod color {
    pub const I: u8 = 51; // Cyan
    pub const O: u8 = 226; // Yellow
    pub const T: u8 = 201; // Magenta
    pub const S: u8 = 46; // Green
    pub const Z: u8 = 196; // Red
    pub const J: u8 = 33; // Blue
    pub const L: u8 = 208; // Orange
    pub const GHOST: u8 = 244;
    pub const BORDER: u8 = 240;
    pub const TEXT: u8 = 255;
    pub const TRASH: u8 = 118;
}

// 7-bag randomizer
pub const BAG_SIZE: usize = 7;
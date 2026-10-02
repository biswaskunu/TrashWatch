use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ServerMsg {
    #[serde(rename = "state")]
    State(GameStateSnapshot),
    #[serde(rename = "chat")]
    Chat { from: String, text: String },
    #[serde(rename = "reaction")]
    Reaction { emoji: String, x: f32, y: f32 },
    #[serde(rename = "welcome")]
    Welcome { room_id: Uuid, your_id: Uuid },
}

#[derive(Serialize, Debug)]
pub struct GameStateSnapshot {
    pub board: [[CellView; 10]; 20],
    pub current_piece: Option<PieceView>,
    pub next_piece: PieceView,
    pub score: u32,
    pub level: u8,
    pub lines_cleared: u32,
    pub trash_streak: u8,
    pub is_paused: bool,
    pub is_game_over: bool,
}

#[derive(Serialize, Debug)]
pub struct CellView {
    pub kind: Option<crate::game::PieceKind>,
    pub is_ghost: bool,
}

#[derive(Serialize, Debug)]
pub struct PieceView {
    pub kind: crate::game::PieceKind,
    pub rotation: u8,
    pub x: i8,
    pub y: i8,
    pub cells: [(i8, i8); 4],
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMsg {
    #[serde(rename = "chat")]
    Chat { text: String },
    #[serde(rename = "reaction")]
    Reaction { emoji: String },
}

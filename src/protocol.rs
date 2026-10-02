use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Debug, Clone)]
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

#[derive(Serialize, Debug, Clone)]
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

#[derive(Serialize, Debug, Clone)]
pub struct CellView {
    pub kind: Option<crate::game::PieceKind>,
    pub is_ghost: bool,
}

#[derive(Serialize, Debug, Clone)]
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

impl From<&crate::game::Cell> for CellView {
    fn from(cell: &crate::game::Cell) -> Self {
        Self { kind: cell.kind, is_ghost: cell.is_ghost }
    }
}

impl From<&crate::game::Piece> for PieceView {
    fn from(piece: &crate::game::Piece) -> Self {
        let shapes = &crate::game::PIECE_SHAPES[piece.kind as usize][piece.rotation as usize];
        let cells: [(i8, i8); 4] = [
            (piece.x + shapes[0].0, piece.y + shapes[0].1),
            (piece.x + shapes[1].0, piece.y + shapes[1].1),
            (piece.x + shapes[2].0, piece.y + shapes[2].1),
            (piece.x + shapes[3].0, piece.y + shapes[3].1),
        ];
        Self {
            kind: piece.kind,
            rotation: piece.rotation,
            x: piece.x,
            y: piece.y,
            cells,
        }
    }
}

impl From<&crate::game::GameState> for GameStateSnapshot {
    fn from(state: &crate::game::GameState) -> Self {
        let board: [[CellView; 10]; 20] = std::array::from_fn(|y| {
            std::array::from_fn(|x| (&state.board[y][x]).into())
        });
        Self {
            board,
            current_piece: state.current_piece.as_ref().map(|p| p.into()),
            next_piece: (&state.next_piece).into(),
            score: state.score,
            level: state.level,
            lines_cleared: state.lines_cleared,
            trash_streak: state.trash_streak,
            is_paused: state.is_paused,
            is_game_over: state.is_game_over,
        }
    }
}

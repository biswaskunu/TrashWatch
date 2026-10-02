use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write};
use crate::{config::*, game::*};

pub struct TerminalRenderer {
    stdout: std::io::Stdout,
    term_width: u16,
    term_height: u16,
    frame: String,
}

impl TerminalRenderer {
    pub fn new() -> anyhow::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        let (term_width, term_height) = terminal::size()?;
        Ok(Self {
            stdout,
            term_width,
            term_height,
            frame: String::new(),
        })
    }

    fn begin_frame(&mut self) {
        self.frame.clear();
        queue!(self.stdout, Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    }

    fn end_frame(&mut self) -> anyhow::Result<()> {
        write!(self.stdout, "{}", self.frame)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn color_for_kind(kind: PieceKind) -> Color {
        Color::AnsiValue(match kind {
            PieceKind::I => color::I,
            PieceKind::O => color::O,
            PieceKind::T => color::T,
            PieceKind::S => color::S,
            PieceKind::Z => color::Z,
            PieceKind::J => color::J,
            PieceKind::L => color::L,
        })
    }

    fn draw_board(&mut self, state: &GameState, board_x: u16, board_y: u16) {
        let ghost = state.ghost_piece();
        let current = state.current_piece;

        let top_left = format!("┌{}┐", "─".repeat(BOARD_WIDTH * 2));
        queue!(self.stdout, cursor::MoveTo(board_x, board_y), Print(&top_left)).unwrap();

        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                let mut cell_char = EMPTY_CHAR;
                let mut cell_color = Color::AnsiValue(color::BORDER);

                let is_ghost = ghost.as_ref().map_or(false, |g| {
                    let shapes = &PIECE_SHAPES[g.kind as usize][g.rotation as usize];
                    shapes.iter().any(|(px, py)| g.x + px == col as i8 && g.y + py == row as i8)
                });

                let is_current = current.as_ref().map_or(false, |p| {
                    let shapes = &PIECE_SHAPES[p.kind as usize][p.rotation as usize];
                    shapes.iter().any(|(px, py)| p.x + px == col as i8 && p.y + py == row as i8)
                });

                if is_current {
                    cell_char = CELL_CHAR;
                    cell_color = Self::color_for_kind(current.unwrap().kind);
                } else if is_ghost {
                    cell_char = GHOST_CHAR;
                    cell_color = Color::AnsiValue(color::GHOST);
                } else if let Some(kind) = state.board[row][col].kind {
                    cell_char = CELL_CHAR;
                    cell_color = Self::color_for_kind(kind);
                }

                queue!(
                    self.stdout,
                    cursor::MoveTo(board_x + 1 + col as u16 * 2, board_y + 1 + row as u16),
                    SetForegroundColor(cell_color),
                    Print(cell_char),
                    SetForegroundColor(Color::Reset)
                ).unwrap();
            }
        }

        let bottom = format!("└{}┘", "─".repeat(BOARD_WIDTH * 2));
        queue!(self.stdout, cursor::MoveTo(board_x, board_y + BOARD_HEIGHT as u16 + 1), Print(&bottom)).unwrap();
    }

    pub fn draw(&mut self, state: &GameState) -> anyhow::Result<()> {
        self.begin_frame();

        let board_pixel_width = (BOARD_WIDTH * 2) as u16 + 2;
        let board_x = (self.term_width.saturating_sub(board_pixel_width)) / 2;
        let board_y = (self.term_height.saturating_sub(BOARD_HEIGHT as u16 + 2)) / 2;

        self.draw_board(state, board_x, board_y);

        self.end_frame()
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

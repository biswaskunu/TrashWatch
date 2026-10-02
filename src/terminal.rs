use crate::{config::*, game::*};
use crossterm::{
    cursor, event, execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    Continue,
    Quit,
    Restart,
}

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
        queue!(
            self.stdout,
            cursor::MoveTo(board_x, board_y),
            Print(&top_left)
        )
        .unwrap();

        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                let mut cell_char = EMPTY_CHAR;
                let mut cell_color = Color::AnsiValue(color::BORDER);

                let is_ghost = ghost.as_ref().is_some_and(|g| {
                    let shapes = &PIECE_SHAPES[g.kind as usize][g.rotation as usize];
                    shapes
                        .iter()
                        .any(|(px, py)| g.x + px == col as i8 && g.y + py == row as i8)
                });

                let is_current = current.as_ref().is_some_and(|p| {
                    let shapes = &PIECE_SHAPES[p.kind as usize][p.rotation as usize];
                    shapes
                        .iter()
                        .any(|(px, py)| p.x + px == col as i8 && p.y + py == row as i8)
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
                )
                .unwrap();
            }
        }

        let bottom = format!("└{}┘", "─".repeat(BOARD_WIDTH * 2));
        queue!(
            self.stdout,
            cursor::MoveTo(board_x, board_y + BOARD_HEIGHT as u16 + 1),
            Print(&bottom)
        )
        .unwrap();
    }

    pub fn draw(&mut self, state: &GameState) -> anyhow::Result<()> {
        self.begin_frame();

        let board_pixel_width = (BOARD_WIDTH * 2) as u16 + 2;
        let board_x = (self.term_width.saturating_sub(board_pixel_width)) / 2;
        let board_y = (self.term_height.saturating_sub(BOARD_HEIGHT as u16 + 2)) / 2;

        self.draw_title(state, board_x, board_y);
        self.draw_board(state, board_x, board_y);
        self.draw_side_panel(state, board_x + board_pixel_width + 2, board_y);

        if state.is_game_over {
            self.draw_overlay(
                "GAME OVER",
                &[
                    &format!("Final Score: {}", state.score),
                    "Press R to restart",
                    "Press Q to quit",
                ],
                board_x,
                board_y,
                board_pixel_width,
            );
        } else if state.is_paused {
            self.draw_overlay(
                "PAUSED",
                &["Press P/Esc to resume", "Press Q to quit"],
                board_x,
                board_y,
                board_pixel_width,
            );
        } else if state.current_piece.is_none() && state.score == 0 && state.lines_cleared == 0 {
            self.draw_overlay(
                "TrashWatch",
                &[
                    "Press any key to start",
                    "",
                    "←/A  Move left    →/D  Move right",
                    "↑/W  Rotate CW     Z    Rotate CCW",
                    "↓/S  Soft drop     Space  Hard drop",
                    "P/Esc  Pause       Q    Quit",
                ],
                board_x,
                board_y,
                board_pixel_width,
            );
        }

        self.end_frame()
    }

    fn draw_overlay(
        &mut self,
        title: &str,
        lines: &[&str],
        board_x: u16,
        board_y: u16,
        board_w: u16,
    ) {
        let overlay_w = 36u16;
        let overlay_h = (lines.len() + 3) as u16;
        let ox = board_x + (board_w.saturating_sub(overlay_w)) / 2;
        let oy = board_y + (BOARD_HEIGHT as u16 + 2).saturating_sub(overlay_h) / 2;

        queue!(
            self.stdout,
            cursor::MoveTo(ox, oy),
            Print(&format!("┌{}┐", "─".repeat(overlay_w as usize - 2)))
        )
        .unwrap();
        queue!(
            self.stdout,
            cursor::MoveTo(ox, oy + 1),
            Print(&format!(
                "│ {:^width$} │",
                title,
                width = overlay_w as usize - 4
            ))
        )
        .unwrap();
        for (i, line) in lines.iter().enumerate() {
            queue!(
                self.stdout,
                cursor::MoveTo(ox, oy + 2 + i as u16),
                Print(&format!(
                    "│ {:<width$} │",
                    line,
                    width = overlay_w as usize - 4
                ))
            )
            .unwrap();
        }
        queue!(
            self.stdout,
            cursor::MoveTo(ox, oy + overlay_h - 1),
            Print(&format!("└{}┘", "─".repeat(overlay_w as usize - 2)))
        )
        .unwrap();
    }

    fn draw_title(&mut self, state: &GameState, board_x: u16, board_y: u16) {
        let title = "TrashWatch";
        let score_str = format!("Score: {}", state.score);
        let title_y = board_y.saturating_sub(2);
        let title_x = board_x + (20 - title.len() as u16) / 2;
        queue!(self.stdout, cursor::MoveTo(title_x, title_y), Print(title)).unwrap();
        let score_x = board_x + 20 + 2;
        queue!(
            self.stdout,
            cursor::MoveTo(score_x, title_y),
            Print(&score_str)
        )
        .unwrap();
    }

    fn draw_side_panel(&mut self, state: &GameState, panel_x: u16, panel_y: u16) {
        let mut y = panel_y;

        queue!(self.stdout, cursor::MoveTo(panel_x, y), Print("Next:")).unwrap();
        y += 1;
        self.draw_next_piece(&state.next_piece, panel_x, y);
        y += 6;

        queue!(
            self.stdout,
            cursor::MoveTo(panel_x, y),
            Print(format!("Level: {}", state.level))
        )
        .unwrap();
        y += 1;
        queue!(
            self.stdout,
            cursor::MoveTo(panel_x, y),
            Print(format!("Lines: {}", state.lines_cleared))
        )
        .unwrap();
        y += 1;
        queue!(
            self.stdout,
            cursor::MoveTo(panel_x, y),
            SetForegroundColor(Color::AnsiValue(color::TRASH)),
            Print(format!("Trash: {}", state.trash_streak)),
            SetForegroundColor(Color::Reset)
        )
        .unwrap();
        y += 2;

        queue!(self.stdout, cursor::MoveTo(panel_x, y), Print("Reactions:")).unwrap();
        y += 1;
        queue!(
            self.stdout,
            cursor::MoveTo(panel_x, y),
            Print("🔥 💀 🚀 ✨")
        )
        .unwrap();
    }

    fn draw_next_piece(&mut self, piece: &Piece, x: u16, y: u16) {
        let shapes = &PIECE_SHAPES[piece.kind as usize][piece.rotation as usize];
        let color = Self::color_for_kind(piece.kind);
        let min_x = shapes.iter().map(|(px, _)| *px).min().unwrap_or(0);
        let max_x = shapes.iter().map(|(px, _)| *px).max().unwrap_or(0);
        let min_y = shapes.iter().map(|(_, py)| *py).min().unwrap_or(0);
        let max_y = shapes.iter().map(|(_, py)| *py).max().unwrap_or(0);
        let piece_w = (max_x - min_x + 1) as u16;
        let piece_h = (max_y - min_y + 1) as u16;
        let _offset_x = (4 - piece_w) / 2;
        let _offset_y = (4 - piece_h) / 2;

        queue!(self.stdout, cursor::MoveTo(x, y), Print("┌────┐")).unwrap();
        for row in 0..4 {
            queue!(self.stdout, cursor::MoveTo(x, y + 1 + row), Print("│")).unwrap();
            for col in 0..4 {
                let mut is_block = false;
                for (px, py) in shapes {
                    if *px - min_x == col as i8 && *py - min_y == row as i8 {
                        is_block = true;
                        break;
                    }
                }
                if is_block {
                    queue!(
                        self.stdout,
                        SetForegroundColor(color),
                        Print(CELL_CHAR),
                        SetForegroundColor(Color::Reset)
                    )
                    .unwrap();
                } else {
                    queue!(self.stdout, Print(EMPTY_CHAR)).unwrap();
                }
            }
            queue!(self.stdout, Print("│")).unwrap();
        }
        queue!(self.stdout, cursor::MoveTo(x, y + 5), Print("└────┘")).unwrap();
    }

    pub fn handle_input(
        &mut self,
        state: &mut GameState,
        _bag: &mut BagRandomizer,
    ) -> anyhow::Result<InputAction> {
        if event::poll(Duration::from_millis(0))? {
            if let event::Event::Key(key) = event::read()? {
                use crossterm::event::{KeyCode, KeyModifiers};

                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Char('Q'), _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return Ok(InputAction::Quit);
                    }
                    (KeyCode::Char('r'), _) | (KeyCode::Char('R'), _) if state.is_game_over => {
                        return Ok(InputAction::Restart);
                    }
                    (KeyCode::Left, _) | (KeyCode::Char('a'), _) | (KeyCode::Char('A'), _) => {
                        state.move_left();
                    }
                    (KeyCode::Right, _) | (KeyCode::Char('d'), _) | (KeyCode::Char('D'), _) => {
                        state.move_right();
                    }
                    (KeyCode::Up, _) | (KeyCode::Char('w'), _) | (KeyCode::Char('W'), _) => {
                        state.rotate_cw();
                    }
                    (KeyCode::Char('z'), _) | (KeyCode::Char('Z'), _) => {
                        state.rotate_ccw();
                    }
                    (KeyCode::Down, _) | (KeyCode::Char('s'), _) | (KeyCode::Char('S'), _) => {
                        state.soft_drop();
                    }
                    (KeyCode::Char(' '), _) => {
                        state.hard_drop();
                    }
                    (KeyCode::Char('p'), _) | (KeyCode::Char('P'), _) | (KeyCode::Esc, _)
                        if !state.is_game_over =>
                    {
                        state.is_paused = !state.is_paused;
                    }
                    _ => {}
                }
            }
        }
        Ok(InputAction::Continue)
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

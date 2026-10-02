use crossterm::{
    cursor, execute,
    style::Color,
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
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

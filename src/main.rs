mod config;
mod game;
mod protocol;
mod spectator;
mod terminal;

use crate::{config::*, game::*, terminal::*};
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    let mut renderer = TerminalRenderer::new()?;
    let mut state = GameState::new();
    let mut bag = BagRandomizer::new();
    let mut last_tick = Instant::now();

    loop {
        match renderer.handle_input(&mut state, &mut bag)? {
            InputAction::Quit => break,
            InputAction::Restart => {
                state = GameState::new();
                bag = BagRandomizer::new();
                last_tick = Instant::now();
            }
            InputAction::Continue => {}
        }

        let now = Instant::now();
        let dt_ms = now.duration_since(last_tick).as_millis() as u32;
        if dt_ms >= TICK_MS {
            state.tick(dt_ms, &mut bag);
            last_tick = now;
        }

        renderer.draw(&state)?;

        std::thread::sleep(Duration::from_millis(1));
    }

    Ok(())
}

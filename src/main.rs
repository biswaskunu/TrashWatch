mod config;
mod game;
mod protocol;
mod spectator;
mod terminal;

use crate::{config::*, game::*, spectator::*, terminal::*};
use axum::{routing::get, serve, Router};
use clap::Parser;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

#[derive(Parser)]
#[command(name = "trashwatch")]
struct Args {
    #[arg(short, long, default_value = "3000")]
    port: u16,
    #[arg(long)]
    spectator: bool,
}

type RoomRegistry = Arc<Mutex<Option<Room>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let registry: RoomRegistry = Arc::new(Mutex::new(None));

    let room_id = create_room(&registry).await;
    println!("Room ID: {}", room_id);
    println!(
        "Spectators: http://localhost:{}/?room={}",
        args.port, room_id
    );

    let broadcast_registry = registry.clone();
    tokio::spawn(async move {
        broadcast_task(broadcast_registry).await;
    });

    let app = Router::new()
        .route("/ws/:room_id", get(ws_handler))
        .nest_service("/", ServeDir::new("."))
        .with_state(registry.clone());

    let listener = TcpListener::bind(("0.0.0.0", args.port)).await?;
    let server_handle = tokio::spawn(async move {
        if let Err(e) = serve(listener, app).await {
            eprintln!("Server error: {}", e);
        }
    });

    let (game_arc, bag_arc) = {
        let reg = registry.lock().await;
        let room = reg.as_ref().unwrap();
        (room.game.clone(), room.bag.clone())
    };

    let mut renderer = TerminalRenderer::new()?;
    let mut last_tick = Instant::now();

    loop {
        let mut state = game_arc.lock().await;
        let mut bag = bag_arc.lock().await;

        match renderer.handle_input(&mut state, &mut bag)? {
            InputAction::Quit => break,
            InputAction::Restart => {
                *state = GameState::new();
                *bag = BagRandomizer::new();
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

        drop(state);
        drop(bag);

        let state = game_arc.lock().await;
        renderer.draw(&state)?;
        drop(state);

        std::thread::sleep(Duration::from_millis(1));
    }

    server_handle.abort();
    Ok(())
}

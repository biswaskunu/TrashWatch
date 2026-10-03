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

async fn game_loop_task(
    game_arc: Arc<Mutex<GameState>>,
    bag_arc: Arc<Mutex<BagRandomizer>>,
    shutdown: Arc<tokio::sync::Notify>,
) {
    let mut last_tick = Instant::now();
    loop {
        tokio::select! {
            _ = shutdown.notified() => break,
            _ = tokio::time::sleep(Duration::from_millis(TICK_MS as u64)) => {
                let now = Instant::now();
                let dt_ms = now.duration_since(last_tick).as_millis() as u32;
                if dt_ms >= TICK_MS {
                    let mut state = game_arc.lock().await;
                    let mut bag = bag_arc.lock().await;
                    state.tick(dt_ms, &mut bag);
                    last_tick = now;
                }
            }
        }
    }
}

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

    let (game_arc, bag_arc) = {
        let reg = registry.lock().await;
        let room = reg.as_ref().unwrap();
        (room.game.clone(), room.bag.clone())
    };

    let shutdown = Arc::new(tokio::sync::Notify::new());
    let game_shutdown = shutdown.clone();
    let game_handle = tokio::spawn(game_loop_task(game_arc.clone(), bag_arc.clone(), game_shutdown));

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

    if args.spectator {
        let url = format!("http://localhost:{}/?room={}", args.port, room_id);
        if let Err(e) = open::that(&url) {
            eprintln!("Failed to open browser: {}", e);
        }
    }

    let mut renderer = TerminalRenderer::new()?;

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        let mut state = game_arc.lock().await;
        let mut bag = bag_arc.lock().await;

        match renderer.handle_input(&mut state, &mut bag)? {
            InputAction::Quit => break,
            InputAction::Restart => {
                *state = GameState::new();
                *bag = BagRandomizer::new();
            }
            InputAction::Continue => {}
        }

        drop(state);
        drop(bag);

        let state = game_arc.lock().await;
        renderer.draw(&state)?;
        drop(state);

        tokio::select! {
            _ = &mut ctrl_c => {
                println!("\nShutting down...");
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(1)) => {}
        }
    }

    shutdown.notify_waiters();
    game_handle.abort();
    server_handle.abort();

    Ok(())
}
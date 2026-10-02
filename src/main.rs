mod config;
mod game;
mod protocol;
mod spectator;
mod terminal;

use crate::{config::*, game::*, spectator::*, terminal::*};
use axum::{response::Html, routing::get, serve, Router};
use clap::Parser;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Parser)]
#[command(name = "trashwatch")]
struct Args {
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

const SPECTATOR_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <title>TrashWatch Spectator</title>
    <style>
        body { font-family: monospace; background: #1a1a1a; color: #fff; padding: 20px; }
        #status { color: #4CAF50; }
        #log { background: #000; padding: 10px; height: 400px; overflow-y: auto; white-space: pre-wrap; }
    </style>
</head>
<body>
    <h1>TrashWatch Spectator</h1>
    <p id="status">Connecting...</p>
    <pre id="log"></pre>
    <script>
        const urlParams = new URLSearchParams(window.location.search);
        const roomId = urlParams.get('room') || prompt('Room ID:');
        if (!roomId) { document.getElementById('status').textContent = 'No room ID provided'; }
        const ws = new WebSocket(`ws://${location.host}/ws/${roomId}`);
        ws.onopen = () => { document.getElementById('status').textContent = 'Connected to ' + roomId; };
        ws.onmessage = (e) => { document.getElementById('log').textContent = e.data + '\n' + document.getElementById('log').textContent; };
        ws.onclose = () => { document.getElementById('status').textContent = 'Disconnected'; };
    </script>
</body>
</html>"#;

async fn serve_spectator_html() -> Html<&'static str> {
    Html(SPECTATOR_HTML)
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
        .route("/", get(serve_spectator_html))
        .route("/ws/:room_id", get(ws_handler))
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

# TrashWatch

Terminal Tetris with live browser spectators.

---

### ( This is a vibe coded project )

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/PRD.md`](docs/PRD.md) | Product Requirements — features, acceptance criteria |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | System architecture — modules, data flow, concurrency |
| [`docs/DESIGN.md`](docs/DESIGN.md) | Detailed design — types, algorithms, protocols, UI |
| [`docs/PHASES.md`](docs/PHASES.md) | Implementation phases — step-by-step plan |
| [`docs/RULES.md`](docs/RULES.md) | Development rules — code style, architecture, git |
| [`docs/API_SPEC.md`](docs/API_SPEC.md) | WebSocket API — message formats, endpoints |
| [`docs/CONFIG.md`](docs/CONFIG.md) | Configuration constants — timing, scoring, colors |
| [`docs/TESTING.md`](docs/TESTING.md) | Testing strategy — unit, integration, property tests |
| [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) | Deployment guide — local, production, Cloudflare Workers |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | Future extensions — post-v1 roadmap |
---

## Quick Start

```bash
# Build and run (terminal game + WebSocket server on :3000)
cargo run

# Opens terminal game; prints Room ID and spectator URL:
#   Spectators: http://localhost:3000/?room=<uuid>

# Run with custom port
cargo run -- --port 8080

# Run and auto-open spectator in browser
cargo run -- --spectator
```

### Controls (Terminal)
| Key | Action |
|-----|--------|
| ← / A | Move left |
| → / D | Move right |
| ↑ / W | Rotate CW |
| Z | Rotate CCW |
| ↓ / S | Soft drop |
| Space | Hard drop |
| P / Esc | Pause |
| R | Restart (game over) |
| Q | Quit |

### Spectator UI
Open `http://localhost:3000` in browser:
- Live board canvas (30px cells, 60fps)
- Next piece preview
- Score/Level/Lines/Trash Streak HUD
- Chat panel (send/receive, message history)
- Reaction buttons: 🔥 💀 🚀 ✨ (float+fade animation)
- Auto-reconnect on connection loss

---

## Project Structure

```
TrashWatch/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry, CLI, Axum server, game loop
│   ├── game.rs          # Pure Tetris logic
│   ├── terminal.rs      # Crossterm renderer + input
│   ├── spectator.rs     # WebSocket handler, room, broadcast
│   ├── protocol.rs      # WS message types (serde)
│   └── config.rs        # Constants
├── spectator.html       # Browser UI (served statically, Phase 5 complete)
├── deploy/
│   └── worker.js        # Cloudflare Workers proxy (Phase 6)
└── docs/                # All documentation
```

---

## Tech Stack

- **Rust (current)** — Axum, Tokio, Tungstenite, Crossterm, Serde, Anyhow, Thiserror, Clap, Rand, UUID, `open`, `tower-http`
- **Browser (current)** — Vanilla JS, Canvas 2D, WebSocket (Phase 5 complete)
- **Deploy (planned)** — Cloudflare Workers (proxy), Fly.io/Render (origin) (Phase 6)

---

## License

MIT

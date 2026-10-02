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
# Build and run (terminal game + server)
cargo run

# Run and auto-open browser spectator
cargo run -- --spectator

# Custom port
cargo run -- --port 8080
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
- Live board canvas
- Next piece preview
- Score/Level/Lines/Trash Streak
- Chat panel
- Reaction buttons: 🔥 💀 🚀 ✨

---

## Project Structure

```
TrashWatch/
├── Cargo.toml
├── src/
│   ├── main.rs          # Entry, CLI, Axum server, game loop
│   ├── game.rs          # Pure Tetris logic
│   ├── terminal.rs      # Crossterm renderer + input
│   ├── spectator.rs     # WebSocket handler, rooms, broadcast
│   ├── protocol.rs      # WS message types (serde)
│   └── config.rs        # Constants
├── spectator.html       # Browser UI (vanilla JS, Canvas)
├── deploy/
│   └── worker.js        # Cloudflare Workers proxy
└── docs/                # All documentation
```

---

## Tech Stack

- **Rust** — Axum, Tokio, Tungstenite, Serde, Crossterm, DashMap, UUID
- **Browser** — Vanilla JS, Canvas 2D, WebSocket
- **Deploy** — Cloudflare Workers (proxy), Fly.io/Render (origin)

---

## License

MIT

# TrashWatch — Implementation Phases

**Version:** 1.0

---

## Phase 0: Project Setup (30 min)
- [ ] Initialize Cargo project
- [ ] Create `Cargo.toml` with all dependencies
- [ ] Create module structure (`src/main.rs`, `game.rs`, `terminal.rs`, `spectator.rs`, `protocol.rs`, `config.rs`)
- [ ] Verify `cargo check` passes

---

## Phase 1: Game Core (2-3 hrs)
- [ ] `PieceKind`, `Piece`, `Cell` types
- [ ] Piece shape tables (7 pieces × 4 rotations)
- [ ] `GameState` struct + `new()`, `spawn_next()`
- [ ] Collision detection
- [ ] Movement: left, right, rotate CW/CCW, soft drop, hard drop
- [ ] Lock delay logic (500ms, 15 resets)
- [ ] Line clear detection + scoring
- [ ] Level progression (every 10 lines)
- [ ] Ghost piece calculation
- [ ] Game over detection
- [ ] Unit tests for core logic

---

## Phase 2: Terminal Renderer (1-2 hrs)
- [ ] `TerminalRenderer` struct + `new()` (raw mode, alt screen)
- [ ] `draw(&mut self, state: &GameState)` — full render
- [ ] Board rendering with colors
- [ ] Ghost piece rendering
- [ ] Next piece preview
- [ ] HUD: Score, Level, Lines, Trash Streak
- [ ] Overlays: Start, Pause, Game Over
- [ ] Input handling loop (crossterm events)
- [ ] Cleanup on Drop

---

## Phase 3: Spectator System (2-3 hrs)
- [ ] `protocol.rs` — `ServerMsg`, `ClientMsg`, `GameStateSnapshot`
- [ ] `Room` struct + `DashMap<Uuid, Arc<Mutex<Room>>>`
- [ ] WS handler: upgrade, join room, send welcome
- [ ] Broadcast task (10Hz interval)
- [ ] Chat message broadcast
- [ ] Reaction handling + overlay management (TTL)
- [ ] Spectator cleanup on disconnect

---

## Phase 4: Main Integration (1-2 hrs)
- [ ] CLI args (`clap`: `--spectator`, `--port`)
- [ ] Axum router: `GET /`, `GET /ws/:room_id`
- [ ] Static file serving for `spectator.html`
- [ ] Game loop task (60Hz) per room
- [ ] Terminal mode: spawn renderer + input loop
- [ ] `--spectator`: open browser via `open` crate
- [ ] Graceful shutdown (Ctrl+C)

---

## Phase 5: Browser UI (2-3 hrs)
- [ ] `spectator.html` — single file, vanilla JS
- [ ] Canvas board rendering (30px cells)
- [ ] Piece rendering with colors matching terminal
- [ ] Ghost piece rendering
- [ ] Next piece preview
- [ ] HUD display
- [ ] Chat panel (send/receive)
- [ ] Reaction buttons (4 emoji)
- [ ] Reaction animation (float + fade)
- [ ] Room ID from URL or auto-generate
- [ ] Auto-reconnect on WS close

---

## Phase 6: Cloudflare Worker (30 min)
- [ ] `deploy/worker.js` — WS proxy + static assets
- [ ] `wrangler.toml` configuration
- [ ] Document deployment steps

---

## Phase 7: Polish & Verify (1 hr)
- [ ] `cargo fmt` + `cargo clippy`
- [ ] Test full flow: `cargo run` → play → open browser → spectate
- [ ] Test `--spectator` flag
- [ ] Test multiple spectators
- [ ] Test game over / restart
- [ ] First commit: `"chore: scaffold TrashWatch — tetris core + ws skeleton"`

---

## Timeline Summary
| Phase | Est. Time | Cumulative |
|-------|-----------|------------|
| 0: Setup | 30 min | 30 min |
| 1: Game Core | 3 hrs | 3.5 hrs |
| 2: Terminal | 2 hrs | 5.5 hrs |
| 3: Spectator | 3 hrs | 8.5 hrs |
| 4: Integration | 2 hrs | 10.5 hrs |
| 5: Browser UI | 3 hrs | 13.5 hrs |
| 6: Worker | 30 min | 14 hrs |
| 7: Polish | 1 hr | 15 hrs |
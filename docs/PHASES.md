# TrashWatch — Implementation Phases

**Version:** 1.0

---

## Phase 0: Project Setup (30 min) ✅ COMPLETE
- [x] Initialize Cargo project
- [x] Create `Cargo.toml` with all dependencies
- [x] Create module structure (`src/main.rs`, `game.rs`, `terminal.rs`, `spectator.rs`, `protocol.rs`, `config.rs`)
- [x] Verify `cargo check` passes

---

## Phase 1: Game Core (2-3 hrs) ✅ COMPLETE
- [x] `PieceKind`, `Piece`, `Cell` types
- [x] Piece shape tables (7 pieces × 4 rotations)
- [x] `GameState` struct + `new()`, `spawn_next()`
- [x] Collision detection
- [x] Movement: left, right, rotate CW/CCW, soft drop, hard drop
- [x] Lock delay logic (500ms, 15 resets)
- [x] Line clear detection + scoring
- [x] Level progression (every 10 lines)
- [x] Ghost piece calculation
- [x] Game over detection
- [x] Unit tests for core logic (14 tests passing)

---

## Phase 2: Terminal Renderer (1-2 hrs) ✅ COMPLETE
- [x] `TerminalRenderer` struct + `new()` (raw mode, alt screen)
- [x] `draw(&mut self, state: &GameState)` — full render
- [x] Board rendering with colors
- [x] Ghost piece rendering
- [x] Next piece preview
- [x] HUD: Score, Level, Lines, Trash Streak
- [x] Overlays: Start, Pause, Game Over
- [x] Input handling loop (crossterm events)
- [x] Cleanup on Drop

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
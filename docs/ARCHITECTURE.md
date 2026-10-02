# TrashWatch — System Architecture

**Version:** 1.0

---

## 1. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        SINGLE BINARY                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Axum HTTP   │  │  Game Loop   │  │  Terminal Renderer   │  │
│  │  + WS Server │◄─┤  (60Hz tick) │──┤  (crossterm, 60Hz)   │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────────────────┘  │
│         │                 │                                      │
│         │    ┌────────────┴────────────┐                         │
│         │    │   Room State (DashMap)  │                         │
│         │    │  - GameState            │                         │
│         │    │  - Spectators: Vec<WS>  │                         │
│         │    │  - Reactions: Vec<>     │                         │
│         │    └────────────┬────────────┘                         │
│         │                 │                                      │
│         ▼                 ▼                                      │
│  ┌──────────────────────────────────────┐                       │
│  │        Broadcast Task (10Hz)         │                       │
│  │  Serialize GameState + Reactions     │                       │
│  │  Send to all WS spectators           │                       │
│  └──────────────────────────────────────┘                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
            ┌─────────────┐      ┌─────────────┐
            │  Terminal   │      │   Browser   │
            │   Player    │      │  Spectator  │
            │  (stdin)    │      │  (WS + UI)  │
            └─────────────┘      └─────────────┘
```

---

## 2. Module Structure

```
src/
├── main.rs          # Entry, CLI, Axum router, game loop spawner
├── game.rs          # Pure game logic (no I/O, no time)
├── terminal.rs      # Crossterm rendering + input handling
├── spectator.rs     # WS handler, room management, broadcast
├── protocol.rs      # Shared WS message types (serde)
└── config.rs        # Constants, tuning parameters
```

---

## 3. Data Flow

### 3.1 Game Tick (60Hz)
```
GameLoop.tick(bag: &mut BagRandomizer)
    │
    ├─► GameState.tick(dt_ms, bag)          // gravity, lock delay, spawn
    │       │
    │       ├─► if current_piece.is_none() → spawn_next(bag)
    │       ├─► Lock delay checked every tick:
    │       │       if on_ground: lock_delay_ms += dt_ms
    │       │       if lock_delay >= 500ms OR lock_resets >= 15 → lock_piece()
    │       ├─► Gravity accumulator (gravity_accum):
    │       │       gravity_accum += dt_ms
    │       │       while gravity_accum >= ms_per_cell:
    │       │           if can_fall: piece.y += 1, lock_delay = 0
    │       │           else if try_lock(): break
    │       │           gravity_accum -= ms_per_cell
    │       └─► Uses GRAVITY_TABLE[level] for ms_per_cell
    │
    ├─► lock_piece() → clear_lines() → add_score() → spawn_next(bag)
    │
    └─► if game_over → broadcast final state, stop loop
```

### 3.2 Input Handling (Terminal)
```
Crossterm event loop
    │
    ├─► Key::Left  → game.move_left()
    ├─► Key::Right → game.move_right()
    ├─► Key::Up    → game.rotate_cw()
    ├─► Key::Down  → game.soft_drop()
    ├─► Key::Char(' ') → game.hard_drop()
    ├─► Key::Char('p') / Esc → toggle_pause()
    └─► Key::Char('r') → restart() (game over)
    └─► Key::Char('q') → quit
```

### 3.3 WebSocket Broadcast (10Hz)
```
BroadcastTask (interval 100ms)
    │
    ├─► For each room in rooms:
    │       │
    │       ├─► Serialize GameStateSnapshot {
    │       │       board, current, next, score, level, lines, trash_streak
    │       │   }
    │       ├─► Serialize Reactions { emoji, x, y, ttl_ms }
    │       └─► Send to all spectator WS (skip closed)
    │
    └─► Clean up expired reactions (ttl <= 0)
```

### 3.4 Spectator Messages (Inbound)
```
WS.on_message(msg)
    │
    ├─► ClientMsg::Chat(text) → broadcast to room spectators
    └─► ClientMsg::Reaction(emoji) → add ReactionOverlay { emoji, x: random, y: top, ttl: 2000 }
```

---

## 4. Concurrency Model

| Component | Concurrency Primitive | Rationale |
|-----------|----------------------|-----------|
| Room Registry | `DashMap<Uuid, Arc<Mutex<Room>>>` | Concurrent read (broadcast) + write (join/leave) |
| Room State | `Mutex<Room>` | Single writer (game loop), multiple readers (WS) |
| Game Loop | Dedicated Tokio task per room | Deterministic 60Hz, no blocking |
| Broadcast | Dedicated Tokio task (global) | Decoupled from game tick |
| WS Connections | One task per connection | Backpressure via bounded channel |
| Terminal | Single task (blocking crossterm) | Crossterm not async-friendly |

---

## 5. State Management

```rust
// Shared, mutable per-room state
struct Room {
    id: Uuid,
    game: GameState,              // Protected by Mutex
    spectators: Vec<Sender<ServerMsg>>,  // Broadcast channels
    reactions: Vec<ReactionOverlay>,
    bag: BagRandomizer,           // 7-bag randomizer per room
}

// GameState (pure, no I/O, Serializable)
#[derive(Clone, Debug, Serialize)]
struct GameState {
    board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    current_piece: Option<Piece>,
    next_piece: Piece,
    score: u32,
    level: u8,
    lines_cleared: u32,
    trash_streak: u8,
    lock_delay_ms: u32,
    lock_resets: u8,
    gravity_accum: u32,           // Gravity time accumulator
    is_paused: bool,
    is_game_over: bool,
}

// Immutable snapshot for broadcast (Send + Sync)
#[derive(Serialize)]
struct GameStateSnapshot {
    board: [[CellView; 10]; 20],
    current_piece: Option<PieceView>,
    next_piece: PieceView,
    score: u32,
    level: u8,
    lines_cleared: u32,
    trash_streak: u8,
    is_paused: bool,
    is_game_over: bool,
}
```

---

## 6. Deployment Architecture

```
Local Dev                          Production (Cloudflare Workers)
─────────────────                  ─────────────────────────────
┌──────────────┐                   ┌──────────────┐
│  cargo run   │                   │  Worker JS   │──► Static Assets (HTML/JS)
│  :3000       │                   │  (proxy)     │
└──────┬───────┘                   └──────┬───────┘
       │                                  │
       │ WS /ws/{room}                    │ WS /ws/{room}
       ▼                                  ▼
┌──────────────┐                   ┌──────────────┐
│  Axum Server │◄──────────────────│  Origin      │
│  (Rust)      │   WSS Proxy       │  (Fly/Render)│
└──────────────┘                   └──────────────┘
```
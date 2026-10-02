# TrashWatch — Development Rules & Conventions

**Version:** 1.0

---

## 1. Code Style

| Rule | Detail |
|------|--------|
| **Formatting** | `cargo fmt` — standard Rust style |
| **Linting** | `cargo clippy -D warnings` — zero warnings |
| **Imports** | Group: std, external, local; alphabetical within group |
| **Naming** | snake_case (vars/fns), PascalCase (types), UPPER_SNAKE (consts) |
| **Async** | `async`/`await` throughout; no `.block_on()` in async ctx |
| **Error Handling** | `anyhow::Result` for main, `thiserror` for domain errors |

---

## 2. Architecture Rules

| Rule | Detail |
|------|--------|
| **Pure Game Logic** | `game.rs` has NO I/O, NO time, NO random — pure functions |
| **Deterministic Tick** | `GameState::tick(dt_ms: u32)` — time passed in, not read |
| **Single Writer** | Game loop is ONLY writer to `GameState` (via Mutex) |
| **Snapshot Broadcast** | Broadcast immutable snapshots, never live refs |
| **No Global State** | All state in `Room` → `DashMap` — testable, swappable |

---

## 3. Terminal Rules

| Rule | Detail |
|------|--------|
| **Crossterm Only** | No direct ANSI escapes — use crossterm API |
| **Alt Screen** | Always enter/exit alternate screen |
| **Cleanup** | `Drop` impl restores terminal (raw mode, cursor, alt screen) |
| **Double Buffer** | Build full frame string, single `write!` per frame |
| **Color** | Use `crossterm::style::Color::AnsiValue(256-color)` |

---

## 4. WebSocket Rules

| Rule | Detail |
|------|--------|
| **Message Format** | JSON with `type` tag (serde `#[serde(tag = "type")]`) |
| **Backpressure** | Bounded channel (64) per spectator; drop slow clients |
| **Heartbeat** | Optional: ping/pong every 30s (Phase 2) |
| **Validation** | Reject unknown message types; limit chat length (200 chars) |

---

## 5. Browser UI Rules

| Rule | Detail |
|------|--------|
| **No Build Step** | Single `spectator.html` — vanilla JS, no bundler |
| **Canvas Only** | No DOM for board — pure Canvas 2D API |
| **RequestAnimationFrame** | Render loop separate from WS receive |
| **Reconnect** | Exponential backoff (1s, 2s, 4s, max 30s) |
| **Accessibility** | Chat: ARIA labels, focus management |

---

## 6. Git & Commit Rules

| Rule | Detail |
|------|--------|
| **Conventional Commits** | `type(scope): subject` |
| **First Commit** | `chore: scaffold TrashWatch — tetris core + ws skeleton` |
| **Feature Commits** | `feat(game): add lock delay`, `feat(term): add ghost piece` |
| **Fix Commits** | `fix(spectator): handle WS reconnect` |
| **No WIP Commits** | Squash before push |

---

## 7. Testing Rules

| Rule | Detail |
|------|--------|
| **Unit Tests** | `game.rs` — pure functions, exhaustive piece tests |
| **Integration** | Manual: `cargo run` + browser |
| **Property Tests** | Optional: `proptest` for collision/rotation |

---

## 8. Dependency Policy

| Policy | Detail |
|--------|--------|
| **Minimize Deps** | Only listed in PRD; audit with `cargo audit` |
| **Version Pinning** | Exact versions in `Cargo.toml` (no `*`) |
| **Features** | Enable only needed features (e.g., `axum` ws, tokio full) |
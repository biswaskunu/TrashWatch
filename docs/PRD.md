# TrashWatch — Product Requirements Document

**Version:** 1.0  
**Date:** 2026-10-02  
**Status:** Approved

---

## 1. Product Overview

**TrashWatch** is a terminal-based Tetris game with live browser spectators. A single player plays in the terminal while any number of spectators watch in real-time via WebSocket-connected browsers. Spectators can send chat messages and emoji reactions that appear as overlays on the game board.

**Core Concept:** "Trash" = cleared lines. CLI output celebrates clears with "Trash" terminology ("+1 Trash!", "Trash Streak: 3").

---

## 2. Target Users

| User Type | Interface | Capabilities |
|-----------|-----------|--------------|
| **Player** | Terminal (crossterm) | Full game control: move, rotate, drop, pause |
| **Spectator** | Browser (HTML/Canvas/WS) | Watch live, chat, send emoji reactions |

---

## 3. Functional Requirements

### 3.1 Game Engine (FR-GAME)
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-GAME-01 | 10×20 board, standard 7 tetrominoes (I,O,T,S,Z,J,L) | P0 |
| FR-GAME-02 | Simple 90° rotation (CW/CCW), no wall kicks | P0 |
| FR-GAME-03 | 500ms lock delay, max 15 resets on move/rotate | P0 |
| FR-GAME-04 | Hard drop (Space), soft drop (Down), left/right, rotate (Up/Z) | P0 |
| FR-GAME-05 | Line clear detection: 1-4 lines, scoring per table below | P0 |
| FR-GAME-06 | Level progression: every 10 lines = +1 level, faster gravity | P0 |
| FR-GAME-07 | Next piece preview (single) | P0 |
| FR-GAME-08 | Ghost piece (landing preview) | P0 |
| FR-GAME-09 | Game over on spawn collision | P0 |
| FR-GAME-10 | Pause/Resume (P or Esc) | P1 |

**Scoring Table:**
| Lines | Base Score | × (Level + 1) |
|-------|------------|---------------|
| 1 | 100 | 100–... |
| 2 | 300 | 300–... |
| 3 | 500 | 500–... |
| 4 | 800 | 800–... |

**Trash Terminology (CLI only):**
- Single clear: `"+1 Trash!"`
- Multi clear: `"+{n} Trash!"`
- Streak (consecutive clears): `"Trash Streak: {n}"`

### 3.2 Terminal UI (FR-TERM)
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-TERM-01 | Raw mode, alternate screen, hidden cursor | P0 |
| FR-TERM-02 | Render board (20×10 cells) with colors per piece | P0 |
| FR-TERM-03 | Render ghost piece (dimmed) | P0 |
| FR-TERM-04 | Side panel: Next piece (4×4), Score, Level, Lines, Trash Streak | P0 |
| FR-TERM-05 | Overlay: "Press SPACE to start" (initial) | P0 |
| FR-TERM-06 | Overlay: "PAUSED" (on pause) | P0 |
| FR-TERM-07 | Overlay: "GAME OVER — Press R to restart, Q to quit" | P0 |
| FR-TERM-08 | Cleanup on exit (restore terminal state) | P0 |

### 3.3 Spectator System (FR-SPEC)
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-SPEC-01 | WebSocket endpoint: `ws://host:3000/ws/{room_id}` | P0 |
| FR-SPEC-02 | Broadcast game state at 10Hz (100ms interval) | P0 |
| FR-SPEC-03 | Room creation: UUID per game, in-memory (DashMap) | P0 |
| FR-SPEC-04 | Spectator join/leave handling | P0 |
| FR-SPEC-05 | Chat: spectators send messages, broadcast to all | P0 |
| FR-SPEC-06 | Reactions: 🔥 💀 🚀 ✨ — click → overlay on board (2s TTL) | P0 |
| FR-SPEC-07 | Serve `spectator.html` at `/` | P0 ✅ |

### 3.4 Browser UI (FR-WEB) ✅ COMPLETE
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-WEB-01 | Canvas 300×600 (30px/cell) — board rendering | P0 ✅ |
| FR-WEB-02 | Next piece preview (120×120) | P0 ✅ |
| FR-WEB-03 | Score/Level/Lines/Trash Streak display | P0 ✅ |
| FR-WEB-04 | Chat panel: message list, input, send button | P0 ✅ |
| FR-WEB-05 | Reaction buttons: 4 emoji, click → send WS | P0 ✅ |
| FR-WEB-06 | Auto-connect to `ws://localhost:3000/ws/{room_id}` | P0 ✅ |
| FR-WEB-07 | Room ID from URL param (`?room=uuid`) or auto-generate | P0 ✅ |

### 3.5 Server & CLI (FR-SRV)
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-SRV-01 | Single binary: `cargo run` → server + terminal game | P0 |
| FR-SRV-02 | `--spectator` flag → open browser to `http://localhost:3000` | P0 |
| FR-SRV-03 | `--port` flag (default 3000) | P0 |
| FR-SRV-04 | Axum HTTP + WS server | P0 |
| FR-SRV-05 | Game loop: 60Hz tick (16.67ms), 10Hz broadcast | P0 |
| FR-SRV-06 | Graceful shutdown (Ctrl+C) | P0 |

### 3.6 Cloudflare Workers (FR-CF)
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-CF-01 | `deploy/worker.js` — proxy WS to origin server | P1 |
| FR-CF-02 | Serve static `spectator.html` from Workers Assets | P1 (Phase 6) |
| FR-CF-03 | Config via `ORIGIN` env var (e.g., `wss://trashwatch.fly.dev`) | P1 |

---

## 4. Non-Functional Requirements

| Category | Requirement |
|----------|-------------|
| **Performance** | 60Hz game tick, 10Hz broadcast, <5ms frame render (terminal) |
| **Latency** | WS broadcast <50ms local, <200ms via Workers proxy |
| **Reliability** | No crashes on spectator join/leave; game continues |
| **Portability** | Linux/macOS/Windows (crossterm), modern browsers (WS, Canvas) |
| **Security** | No auth (local dev); input validation on WS messages |
| **Maintainability** | Modular crate structure, pure game logic, no global state |

---

## 5. Out of Scope (v1)
- Multiple simultaneous games (rooms) — **single room only**
- Spectator-to-spectator chat — **broadcast only**
- Replay system
- High score persistence
- Mobile touch controls
- TETR.IO / Jstris compatibility
- SRS rotation / wall kicks
- Hold piece
- Bag randomizer (7-bag) — **simple random for v1**

---

## 6. Acceptance Criteria

| Scenario | Expected |
|----------|----------|
| `cargo run` | Server starts, terminal opens, "Press SPACE to start" |
| SPACE pressed | Game begins, pieces fall, controls work |
| Line cleared | CLI prints "+1 Trash!" / "Trash Streak: 2" |
| Browser opens `localhost:3000` | Shows live board, next, score, chat, reactions |
| Click 🔥 | Emoji appears on board for 2s on all spectators |
| Type chat + Enter | Message appears in all spectator chat panels |
| Game over | Terminal shows "GAME OVER", browser shows final state |
| `cargo run -- --spectator` | Opens browser automatically |
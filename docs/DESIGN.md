# TrashWatch — Detailed Design

**Version:** 1.2 (Updated to match Phase 1-2 implementation)

---

> **Status:** Sections 1-2 (Game Logic, Terminal Renderer) are **implemented**.
> Sections 3-5 (Spectator Protocol, Browser UI, Cloudflare Worker) are **planned** for Phases 3-6.

---

## 1. Game Logic Design (`src/game.rs`)

### 1.1 Core Types
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub enum PieceKind { I, O, T, S, Z, J, L }

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
pub struct Cell {
    pub kind: Option<PieceKind>,
    pub is_ghost: bool,
}

impl Cell {
    pub fn empty() -> Self { Self { kind: None, is_ghost: false } }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Piece {
    pub kind: PieceKind,
    pub rotation: u8,      // 0-3
    pub x: i8,             // Board column (0-9), spawn at 3
    pub y: i8,             // Board row (0-19), spawn at 0 (top)
}

#[derive(Clone, Debug, Serialize)]
pub struct GameState {
    pub board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current_piece: Option<Piece>,
    pub next_piece: Piece,
    pub score: u32,
    pub level: u8,
    pub lines_cleared: u32,
    pub trash_streak: u8,
    pub lock_delay_ms: u32,
    pub lock_resets: u8,
    pub gravity_accum: u32,
    pub is_paused: bool,
    pub is_game_over: bool,
}
```

### 1.2 Piece Definitions (Simple 90° Rotation)
Each piece has 4 rotations defined as `[(x, y); 4]` offsets from origin.
Uses `PIECE_SHAPES: [[[(i8, i8); 4]; 4]; 7]` — 7 pieces × 4 rotations × 4 blocks.
O-piece uses identical shapes for all 4 rotations.

```rust
impl PieceKind {
    pub fn shapes(&self) -> &[[(i8, i8); 4]; 4] {
        &PIECE_SHAPES[*self as usize]
    }
}
```

### 1.3 7-Bag Randomizer
```rust
pub struct BagRandomizer {
    bag: Vec<PieceKind>,
    rng: StdRng,
}

impl BagRandomizer {
    pub fn new() -> Self { /* shuffles all 7 pieces */ }
    pub fn next(&mut self) -> PieceKind { /* pop or refill */ }
}
```

### 1.4 Key Algorithms

**Collision Check:**
```rust
fn collides(&self, piece: &Piece, dx: i8, dy: i8, drot: i8) -> bool {
    let rot = ((piece.rotation as i8 + drot) & 3) as usize;
    for (px, py) in self.kind_shapes(piece.kind)[rot] {
        let x = piece.x + px + dx;
        let y = piece.y + py + dy;
        if x < 0 || x >= BOARD_WIDTH as i8 || y >= BOARD_HEIGHT as i8 { return true; }
        if y >= 0 && self.board[y as usize][x as usize].kind.is_some() { return true; }
    }
    false
}
```

**Line Clear (handles multi-line clears correctly):**
```rust
fn clear_lines(&mut self) -> u8 {
    let mut cleared = 0;
    let mut y = BOARD_HEIGHT;
    while y > 0 {
        y -= 1;
        if self.board[y].iter().all(|c| c.kind.is_some()) {
            cleared += 1;
            for yy in (1..=y).rev() {
                self.board[yy] = self.board[yy - 1];
            }
            self.board[0] = [Cell::empty(); BOARD_WIDTH];
            y += 1; // Re-check this index after shift
        }
    }
    cleared
}
```

**Lock Delay (checked every tick + max resets):**
```rust
const LOCK_DELAY_MS: u32 = 500;
const MAX_LOCK_RESETS: u8 = 15;

pub fn tick(&mut self, dt_ms: u32, bag: &mut BagRandomizer) {
    if self.is_paused || self.is_game_over { return; }

    if self.current_piece.is_none() {
        self.spawn_next(bag);
        return;
    }

    // Lock delay checked independently every tick
    if let Some(piece) = self.current_piece {
        if self.collides(&piece, 0, 1, 0) {
            self.lock_delay_ms += dt_ms;
            if self.try_lock() { return; }
        } else {
            self.lock_delay_ms = 0;
        }
    }

    // Gravity accumulator
    let frames_per_cell = GRAVITY_TABLE[self.level.min(29) as usize];
    let ms_per_cell = frames_per_cell * TICK_MS;
    self.gravity_accum += dt_ms;
    while self.gravity_accum >= ms_per_cell {
        if let Some(piece) = self.current_piece {
            let can_fall = !self.collides(&piece, 0, 1, 0);
            if can_fall {
                self.current_piece.as_mut().unwrap().y += 1;
                self.lock_delay_ms = 0;
            } else if self.try_lock() {
                self.gravity_accum = 0;
                break;
            }
        }
        self.gravity_accum -= ms_per_cell;
    }
}

fn try_lock(&mut self) -> bool {
    if self.lock_delay_ms >= LOCK_DELAY_MS || self.lock_resets >= MAX_LOCK_RESETS {
        self.lock_piece();
        true
    } else { false }
}
```

**Scoring & Level:**
```rust
fn add_score(&mut self, lines: u8) {
    let base = match lines {
        1 => SCORE_SINGLE, 2 => SCORE_DOUBLE,
        3 => SCORE_TRIPLE, 4 => SCORE_QUAD, _ => 0,
    };
    self.score += base * (self.level as u32 + 1);
    self.lines_cleared += lines as u32;
    self.level = (self.lines_cleared / LINES_PER_LEVEL) as u8;
    self.trash_streak = self.trash_streak.saturating_add(1);
}
```

**Ghost Piece:**
```rust
pub fn ghost_piece(&self) -> Option<Piece> {
    let mut ghost = self.current_piece?;
    while !self.collides(&ghost, 0, 1, 0) { ghost.y += 1; }
    Some(ghost)
}
```

---

## 2. Terminal Renderer Design (`src/terminal.rs`)

### 2.1 Color Scheme (ANSI 256-color)
| Element | Color | Code |
|---------|-------|------|
| I | Cyan | 51 |
| O | Yellow | 226 |
| T | Magenta | 201 |
| S | Green | 46 |
| Z | Red | 196 |
| J | Blue | 33 |
| L | Orange | 208 |
| Ghost | Dim white | 244 |
| Board border | Gray | 240 |
| UI text | White | 255 |
| Trash streak | Bright green | 118 |

### 2.2 Render Layout
```
┌────────────────────────────────────────┐
│  TrashWatch          Score: 123400     │
│  ┌──────────────┐  Next:    Level: 5   │
│  │              │  ┌────┐   Lines: 42  │
│  │   BOARD      │  │ [] │   Trash: 3   │
│  │  (20×10)     │  └────┘              │
│  │              │                       │
│  └──────────────┘  [🔥][💀][🚀][✨]     │
└────────────────────────────────────────┘
```

### 2.3 Input Mapping
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

---

## 3. Spectator Protocol Design (`src/protocol.rs`)

### 3.1 Server → Client (10Hz)
```rust
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum ServerMsg {
    #[serde(rename = "state")]
    State(GameStateSnapshot),
    #[serde(rename = "chat")]
    Chat { from: String, text: String },
    #[serde(rename = "reaction")]
    Reaction { emoji: String, x: f32, y: f32 },
    #[serde(rename = "welcome")]
    Welcome { room_id: Uuid, your_id: Uuid },
}

#[derive(Serialize, Debug)]
pub struct GameStateSnapshot {
    pub board: [[CellView; 10]; 20],
    pub current_piece: Option<PieceView>,
    pub next_piece: PieceView,
    pub score: u32,
    pub level: u8,
    pub lines_cleared: u32,
    pub trash_streak: u8,
    pub is_paused: bool,
    pub is_game_over: bool,
}

#[derive(Serialize, Debug)]
pub struct CellView {
    pub kind: Option<PieceKind>,
    pub is_ghost: bool,
}

#[derive(Serialize, Debug)]
pub struct PieceView {
    pub kind: PieceKind,
    pub rotation: u8,
    pub x: i8,
    pub y: i8,
    pub cells: [(i8, i8); 4],
}
```

### 3.2 Client → Server
```rust
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMsg {
    #[serde(rename = "chat")]
    Chat { text: String },
    #[serde(rename = "reaction")]
    Reaction { emoji: String },
}
```

---

## 4. Browser UI Design (`spectator.html`)

### 4.1 Canvas Rendering
- **Board:** 300×600px (30px/cell), origin top-left
- **Piece colors:** Match terminal ANSI palette (CSS variables)
- **Ghost:** 30% opacity
- **Reactions:** Absolute position on canvas, fade out over 2s

### 4.2 Layout
```
┌─────────────────────────────────────────────────────────────┐
│  TrashWatch — Room: abc-123...                    [Copy URL] │
├──────────────────┬──────────────────────┬───────────────────┤
│                  │                      │  Next Piece       │
│   CANVAS         │   CHAT               │  ┌────────────┐   │
│   (300×600)      │   ┌──────────────┐   │  │            │   │
│                  │   │ Messages...  │   │  │  [Piece]   │   │
│                  │   ├──────────────┤   │  └────────────┘   │
│                  │   │ [Input] [Send]│  │                   │
│                  │   └──────────────┘   │  Score: 123,400   │
│                  │                      │  Level: 5         │
│                  │  REACTIONS           │  Lines: 42        │
│                  │  [🔥] [💀] [🚀] [✨]  │  Trash Streak: 3  │
└──────────────────┴──────────────────────┴───────────────────┘
```

### 4.3 JS Architecture
```javascript
const ws = new WebSocket(`ws://${location.host}/ws/${roomId}`);
ws.onmessage = (e) => {
  const msg = JSON.parse(e.data);
  switch (msg.type) {
    case 'state': renderState(msg); break;
    case 'chat': appendChat(msg); break;
    case 'reaction': spawnReaction(msg); break;
  }
};

function renderState(s) {
  ctx.clearRect(0, 0, 300, 600);
  drawBoard(s.board);
  if (s.current_piece) drawPiece(s.current_piece, false);
  drawGhost(s.current_piece);
  drawNext(s.next_piece);
  updateHUD(s);
}

function animateReactions() {
  reactions.forEach(r => {
    r.y -= 0.5;
    r.alpha -= 0.02;
    drawEmoji(r.emoji, r.x, r.y, r.alpha);
  });
  reactions = reactions.filter(r => r.alpha > 0);
  requestAnimationFrame(animateReactions);
}
```

---

## 5. Cloudflare Worker Design (`deploy/worker.js`)

```javascript
export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    
    if (url.pathname.startsWith('/ws/')) {
      return proxyWebSocket(request, env.ORIGIN);
    }
    return env.ASSETS.fetch(request);
  }
};

async function proxyWebSocket(request, origin) {
  const upgradeHeader = request.headers.get('Upgrade');
  if (upgradeHeader !== 'websocket') {
    return new Response('Expected WebSocket', { status: 400 });
  }
  
  const [client, server] = Object.values(new WebSocketPair());
  
  const originUrl = new URL(request.url);
  originUrl.host = new URL(origin).host;
  originUrl.protocol = 'wss:';
  
  const originWs = await fetch(originUrl.toString(), { headers: request.headers });
  
  pump(client, originWs);
  pump(originWs, client);
  
  return new Response(null, { status: 101, webSocket: client });
}

function pump(from, to) {
  const reader = from.readable.getReader();
  const writer = to.writable.getWriter();
  reader.pipeTo(writer);
}
```
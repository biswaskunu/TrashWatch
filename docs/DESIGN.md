# TrashWatch — Detailed Design

**Version:** 1.0

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

#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub kind: PieceKind,
    pub rotation: u8,      // 0-3
    pub x: i8,             // Board column (0-9), spawn at 3
    pub y: i8,             // Board row (0-19), spawn at 0 (top)
}

#[derive(Clone, Debug, Serialize)]
pub struct GameState {
    pub board: [[Cell; 10]; 20],
    pub current_piece: Option<Piece>,
    pub next_piece: Piece,
    pub score: u32,
    pub level: u8,
    pub lines_cleared: u32,
    pub trash_streak: u8,
    pub lock_delay_ms: u32,
    pub lock_resets: u8,
    pub is_paused: bool,
    pub is_game_over: bool,
}
```

### 1.2 Piece Definitions (Simple 90° Rotation)
Each piece has 4 rotations defined as `[(x, y); 4]` offsets from origin.

```rust
const PIECE_SHAPES: [&[[(i8, i8); 4]; 4]; 7] = [
    // I
    &[[(0,0),(1,0),(2,0),(3,0)], [(2,0),(2,1),(2,2),(2,3)], ...],
    // O (all same)
    &[[(0,0),(1,0),(0,1),(1,1)]; 4],
    // T, S, Z, J, L ...
];
```

### 1.3 Key Algorithms

**Collision Check:**
```rust
fn collides(&self, piece: &Piece, dx: i8, dy: i8, drot: i8) -> bool {
    let rot = (piece.rotation + drot) & 3;
    for (px, py) in SHAPES[piece.kind][rot] {
        let x = piece.x + px + dx;
        let y = piece.y + py + dy;
        if x < 0 || x >= 10 || y >= 20 { return true; }
        if y >= 0 && self.board[y as usize][x as usize].kind.is_some() { return true; }
    }
    false
}
```

**Line Clear:**
```rust
fn clear_lines(&mut self) -> u8 {
    let mut cleared = 0;
    for y in (0..20).rev() {
        if self.board[y].iter().all(|c| c.kind.is_some()) {
            cleared += 1;
            // Shift down
            for yy in (1..=y).rev() {
                self.board[yy] = self.board[yy - 1];
            }
            self.board[0] = [Cell::empty(); 10];
        }
    }
    cleared
}
```

**Lock Delay:**
```rust
const LOCK_DELAY_MS: u32 = 500;
const MAX_LOCK_RESETS: u8 = 15;

fn tick(&mut self, dt_ms: u32) {
    if self.is_paused || self.is_game_over { return; }
    
    if let Some(piece) = self.current_piece {
        // Try gravity
        if !self.collides(piece, 0, 1, 0) {
            self.current_piece.as_mut().unwrap().y += 1;
            self.lock_delay_ms = 0;
        } else {
            self.lock_delay_ms += dt_ms;
            if self.lock_delay_ms >= LOCK_DELAY_MS {
                self.lock_piece();
            }
        }
    } else {
        self.spawn_next();
    }
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
    pub cells: [(i8, i8); 4],  // Precomputed for rendering
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
    Reaction { emoji: String },  // One of: "🔥", "💀", "🚀", "✨"
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
// Single-file vanilla JS (no build step)
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
    r.y -= 0.5;  // Float up
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
// deploy/worker.js
export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    
    // WebSocket proxy
    if (url.pathname.startsWith('/ws/')) {
      return proxyWebSocket(request, env.ORIGIN);
    }
    
    // Static assets (spectator.html, etc.)
    return env.ASSETS.fetch(request);
  }
};

async function proxyWebSocket(request, origin) {
  const upgradeHeader = request.headers.get('Upgrade');
  if (upgradeHeader !== 'websocket') {
    return new Response('Expected WebSocket', { status: 400 });
  }
  
  const [client, server] = Object.values(new WebSocketPair());
  
  // Connect to origin
  const originUrl = new URL(request.url);
  originUrl.host = new URL(origin).host;
  originUrl.protocol = 'wss:';
  
  const originWs = await fetch(originUrl.toString(), {
    headers: request.headers,
  });
  
  // Bidirectional piping
  pump(client, originWs);
  pump(originWs, client);
  
  return new Response(null, { status: 101, webSocket: client });
}

function pump(from, to) {
  // WebSocketStream piping (simplified)
  const reader = from.readable.getReader();
  const writer = to.writable.getWriter();
  reader.pipeTo(writer);
}
```
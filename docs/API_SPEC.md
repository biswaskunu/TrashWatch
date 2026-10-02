# TrashWatch — WebSocket API Specification

**Version:** 1.0  
**Base URL:** `ws://localhost:3000/ws/{room_id}`

---

## 1. Connection

```
GET /ws/{room_id} HTTP/1.1
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Version: 13
Sec-WebSocket-Key: <base64>
```

- `room_id`: UUID v4 (e.g., `550e8400-e29b-41d4-a716-446655440000`)
- Server accepts, creates room if not exists
- Server sends `Welcome` message immediately

---

## 2. Message Format

All messages: JSON with `type` discriminant.

---

## 3. Server → Client Messages

### 3.1 Welcome (once on connect)
```json
{
  "type": "welcome",
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "your_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
}
```

### 3.2 State (10Hz broadcast)
```json
{
  "type": "state",
  "board": [
    [null, null, ..., null],
    ...
    [{"kind": "T", "is_ghost": false}, ...]
  ],
  "current_piece": {
    "kind": "I",
    "rotation": 0,
    "x": 3,
    "y": 0,
    "cells": [[0,0],[1,0],[2,0],[3,0]]
  },
  "next_piece": {
    "kind": "L",
    "rotation": 0,
    "x": 0,
    "y": 0,
    "cells": [[0,0],[0,1],[0,2],[1,2]]
  },
  "score": 123400,
  "level": 5,
  "lines_cleared": 42,
  "trash_streak": 3,
  "is_paused": false,
  "is_game_over": false
}
```

**Notes:**
- `board`: 20 rows × 10 cols, bottom-up (index 0 = bottom row)
- `null` = empty cell
- `cells`: Precomputed absolute offsets for rendering

### 3.3 Chat Message
```json
{
  "type": "chat",
  "from": "Spectator#1234",
  "text": "Nice clear!"
}
```

### 3.4 Reaction Overlay
```json
{
  "type": "reaction",
  "emoji": "🔥",
  "x": 4.5,
  "y": 18.0
}
```
- `x`, `y`: Board coordinates (0-9, 0-19), center of cell
- Client renders at position, floats up, fades over 2s

---

## 4. Client → Server Messages

### 4.1 Chat
```json
{
  "type": "chat",
  "text": "Hello spectators!"
}
```
- Max 200 chars, sanitized server-side

### 4.2 Reaction
```json
{
  "type": "reaction",
  "emoji": "🚀"
}
```
- Valid emoji: `🔥`, `💀`, `🚀`, `✨` (others ignored)

---

## 5. HTTP Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Serve `spectator.html` |
| GET | `/ws/{room_id}` | WebSocket upgrade |

---

## 6. Error Handling

| Scenario | Server Behavior |
|----------|-----------------|
| Invalid room_id | 404 on HTTP, WS close 4004 |
| Malformed JSON | WS close 4000 |
| Unknown message type | Ignore (log warning) |
| Chat too long | Truncate to 200 chars |
| Invalid emoji | Ignore |
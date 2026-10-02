# TrashWatch — Testing Strategy

**Version:** 1.0

---

## 1. Unit Tests (`src/game.rs`)

| Test | Description |
|------|-------------|
| `test_piece_spawn_position` | Piece spawns at (3, 0) |
| `test_collision_left_wall` | Cannot move past x=0 |
| `test_collision_right_wall` | Cannot move past x=9 |
| `test_collision_floor` | Cannot move past y=19 |
| `test_collision_locked_piece` | Cannot overlap placed pieces |
| `test_rotation_cw_ccw` | 4 rotations = identity |
| `test_rotation_bounds` | Rotation respects walls (no kicks) |
| `test_line_clear_single` | 1 line → 100 pts |
| `test_line_clear_double` | 2 lines → 300 pts |
| `test_line_clear_triple` | 3 lines → 500 pts |
| `test_line_clear_quad` | 4 lines → 800 pts |
| `test_level_progression` | 10 lines → level 1 |
| `test_lock_delay` | 500ms delay before lock |
| `test_lock_resets` | Move/rotate resets delay (max 15) |
| `test_ghost_position` | Ghost at lowest valid Y |
| `test_game_over_on_spawn` | Spawn collision = game over |

---

## 2. Integration Tests (Manual)

| Scenario | Steps | Expected |
|----------|-------|----------|
| **Basic Play** | `cargo run` → SPACE → play 1 min | No crashes, smooth 60Hz |
| **Line Clear** | Clear 1 line | CLI: "+1 Trash!" |
| **Trash Streak** | Clear 3 lines consecutively | CLI: "Trash Streak: 3" |
| **Spectator Join** | Open browser during game | Sees live board |
| **Spectator Chat** | Type in browser, send | Appears in all browsers |
| **Spectator Reaction** | Click 🔥 | Emoji floats on all browsers |
| **Multiple Spectators** | Open 3 browser tabs | All in sync |
| **Game Over** | Fill to top | Terminal: "GAME OVER", browser shows final |
| **Restart** | Press R on game over | New game starts |
| **Pause** | Press P | Game pauses, "PAUSED" overlay |
| **--spectator Flag** | `cargo run -- --spectator` | Browser opens auto |

---

## 3. Property Tests (Optional)

```rust
// proptest: random piece + random board → collision() never panics
// proptest: rotation 4x = identity for all pieces
// proptest: lock_piece() always produces valid board state
```
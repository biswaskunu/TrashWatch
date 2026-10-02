# TrashWatch — Testing Strategy

**Version:** 1.1 (Updated to match implementation)

---

## 1. Unit Tests (`src/game.rs`)

| Test | Description |
|------|-------------|
| `test_piece_shapes_valid` | All 7 pieces × 4 rotations have 4 blocks |
| `test_collision_bounds` | Left/right/bottom wall collisions |
| `test_collision_board` | Collision with locked pieces on board |
| `test_movement_left_right` | Left/right movement works |
| `test_movement_rotate_cw_ccw` | CW/CCW rotation works |
| `test_soft_drop` | Soft drop moves down, awards 1 pt |
| `test_hard_drop` | Hard drop locks piece, returns 2 pts/cell |
| `test_lock_delay_resets_on_move` | Move/rotate resets lock_delay_ms, increments lock_resets |
| `test_lock_delay_forces_lock_after_15` | 15 resets forces piece lock |
| `test_line_clear_scoring_1_2_3_4` | 1/2/3/4 line clears score correctly |
| `test_level_progression_every_10` | Level up every 10 lines |
| `test_trash_streak_increment_reset` | Streak +1 on clear, reset on non-clear |
| `test_game_over_on_spawn_collision` | Spawn collision = game over |
| `test_ghost_piece_at_lock_position` | Ghost piece at lock position |

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

---

## 4. Verification Gates (CI-ready)

```bash
cargo fmt                    # Formatting
cargo check                  # Type check
cargo test                   # All 14 unit tests pass
cargo clippy -D warnings     # Zero warnings
```
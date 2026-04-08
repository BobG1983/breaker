# Protocol: Siphon

## Category
`custom-system`

## Game Design
You WANT to set up multi-kill chains to farm time off the clock.

- Kill a cell: 2s streak window starts.
- Each subsequent kill within the window: +0.5s added to node timer. Window resets to 2s.
- First kill starts the streak but adds no time.
- 2s without a kill: streak ends.
- All kill sources count (bolt, AoE, chain lightning, explosions).
- Values (window duration, time per kill) are tunable.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct SiphonConfig {
    /// Duration of the kill streak window in seconds. Resets on each kill.
    pub streak_window: f32,
    /// Seconds added to the node timer for each kill after the first in a streak.
    pub time_per_kill: f32,
}
```

## Components
```rust
/// Tracks the active kill streak for Siphon. Global resource (not per-bolt, since all
/// kill sources count).
#[derive(Resource, Debug, Default)]
pub(crate) struct SiphonStreak {
    /// Time remaining in the current streak window. 0.0 = no active streak.
    pub window_remaining: f32,
    /// Number of kills in the current streak (first kill = 1, adds no time).
    pub kill_count: u32,
}
```

## Messages
**Reads**: `CellDestroyedAt { position, was_required_to_clear }`
**Sends**: `ReverseTimePenalty { seconds }` (adds time back to the node timer)

**Convention**: Siphon uses `ReverseTimePenalty` to add time to the node timer. This is the existing message that adds `seconds` back to `NodeTimer::remaining` (clamped to `NodeTimer::total`). This is semantically correct: the protocol reverses the time pressure by returning time to the clock.

## Systems

### `siphon_on_cell_destroyed`
- **Schedule**: `FixedUpdate`
- **Run if**: `protocol_active(ProtocolKind::Siphon)` + `in_state(NodeState::Playing)`
- **What it does**:
  1. Reads `CellDestroyedAt` messages.
  2. For each cell kill:
     a. If no active streak (`window_remaining <= 0.0`): start a new streak. Set `kill_count = 1`, `window_remaining = config.streak_window`. No time added (first kill starts the streak).
     b. If active streak (`window_remaining > 0.0`): increment `kill_count`, reset `window_remaining = config.streak_window`, send `ReverseTimePenalty { seconds: config.time_per_kill }`.
  3. Multiple kills in the same frame each increment individually and each (after the first) add time.
- **Ordering**: After cell destruction systems (so `CellDestroyedAt` has been sent). Before `NodeSystems::ApplyTimePenalty` (so added time is factored into the same frame's timer tick).

### `siphon_tick_streak`
- **Schedule**: `FixedUpdate`
- **Run if**: `protocol_active(ProtocolKind::Siphon)` + `in_state(NodeState::Playing)`
- **What it does**:
  1. If `streak.window_remaining > 0.0`: subtract `time.delta_secs()` from `window_remaining`.
  2. If `window_remaining` drops to `<= 0.0`: reset streak (`kill_count = 0`, `window_remaining = 0.0`).
- **Ordering**: Runs every tick. Before `siphon_on_cell_destroyed` so the window check happens on fresh delta, then kills within the same frame can extend it.

### `siphon_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)` or `OnEnter(NodeState::Transitioning)`
- **What it does**: Resets `SiphonStreak` to default (no active streak). Streaks do not persist across nodes.

## Cross-Domain Dependencies
- **cells**: Reads `CellDestroyedAt` message. All kill sources (bolt, AoE, chain lightning, explosions) already funnel through the cell destruction pipeline and emit this message.
- **run/node**: Sends `ReverseTimePenalty` message to add time to the node timer. Owned by `state/run/node`.
- **shared**: Reads `Time` resource for delta time (streak window countdown).

## Expected Behaviors (for test specs)

1. **First kill starts streak but adds no time**
   - Given: Siphon active, `SiphonStreak::default()` (no active streak), `config.streak_window: 2.0`, `config.time_per_kill: 0.5`
   - When: `CellDestroyedAt` is sent (first kill)
   - Then: `SiphonStreak { window_remaining: 2.0, kill_count: 1 }`. No `ReverseTimePenalty` sent.

2. **Second kill within window adds time**
   - Given: `SiphonStreak { window_remaining: 1.5, kill_count: 1 }`, `config.time_per_kill: 0.5`
   - When: `CellDestroyedAt` is sent (second kill)
   - Then: `SiphonStreak { window_remaining: 2.0, kill_count: 2 }`. `ReverseTimePenalty { seconds: 0.5 }` sent.

3. **Third and subsequent kills each add time**
   - Given: `SiphonStreak { window_remaining: 1.8, kill_count: 2 }`, `config.time_per_kill: 0.5`
   - When: `CellDestroyedAt` is sent (third kill)
   - Then: `SiphonStreak { window_remaining: 2.0, kill_count: 3 }`. `ReverseTimePenalty { seconds: 0.5 }` sent.

4. **Window resets on each kill**
   - Given: `SiphonStreak { window_remaining: 0.1, kill_count: 3 }`, `config.streak_window: 2.0`
   - When: `CellDestroyedAt` is sent
   - Then: `window_remaining` resets to `2.0` (not added to remaining — hard reset).

5. **Streak expires after window elapses**
   - Given: `SiphonStreak { window_remaining: 0.5, kill_count: 3 }`, delta_secs = 0.6
   - When: `siphon_tick_streak` runs
   - Then: `SiphonStreak { window_remaining: 0.0, kill_count: 0 }`. Streak ended.

6. **Kill after expired streak starts new streak (no time added)**
   - Given: `SiphonStreak { window_remaining: 0.0, kill_count: 0 }`
   - When: `CellDestroyedAt` is sent
   - Then: New streak: `SiphonStreak { window_remaining: 2.0, kill_count: 1 }`. No `ReverseTimePenalty` sent.

7. **Multiple kills in same frame each count**
   - Given: `SiphonStreak { window_remaining: 0.0, kill_count: 0 }`, 3 cells destroyed in same frame
   - When: 3 `CellDestroyedAt` messages processed in sequence
   - Then: First kill starts streak (no time). Second kill adds 0.5s. Third kill adds 0.5s. Total time added: 1.0s. `kill_count: 3`.

8. **Streak cleared on node end**
   - Given: `SiphonStreak { window_remaining: 1.5, kill_count: 4 }`
   - When: Node transitions out of `NodeState::Playing`
   - Then: `SiphonStreak` reset to default.

9. **Added time clamped to NodeTimer::total**
   - Given: Node timer at 58.0s out of 60.0s total, `config.time_per_kill: 0.5`
   - When: Kill within active streak sends `ReverseTimePenalty { seconds: 0.5 }`
   - Then: Node timer becomes 58.5s (clamped by `reverse_time_penalty` system, not by Siphon). If timer was already at 60.0s, no effect.

## Edge Cases
- **AoE destroying many cells at once**: Each destruction event counts as a separate kill. A shockwave killing 8 cells produces 8 `CellDestroyedAt` messages. The first starts/continues the streak, subsequent ones each add time (if streak is active from a prior kill or the first of this batch).
- **Time overflow**: `ReverseTimePenalty` handler in `run/node` clamps to `NodeTimer::total`. Siphon never overflows.
- **No cells left to kill**: Streak naturally expires after `streak_window` seconds with no kills. Clean timeout.
- **Node timer already expired**: If the timer hits 0 and `TimerExpired` fires, the node is lost regardless of Siphon's streak. Adding time after expiry has no effect (game state transitions).
- **Interaction with Deadline**: Both can be active. Siphon adds time, Deadline rewards low time. They create a tension: Siphon extends the timer (reducing Deadline's upside) but keeps you alive longer to benefit from Deadline when it does kick in. No mechanical conflict — just strategic tension.

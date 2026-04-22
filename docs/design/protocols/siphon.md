# Protocol: Siphon

## Category
`custom-system`

## Game Design
You WANT to set up multi-kill chains to farm time off the clock.

- Kill a cell: 2s streak window starts.
- Each subsequent kill within the window adds time to the node timer — **escalating** per streak position (`time_per_kill * (N - 1)` for the Nth kill). The first kill starts the streak and adds no time.
- Window resets to 2s on every kill.
- 2s without a kill: streak ends.
- All kill sources count (bolt, shockwave, chain lightning, explosions — anything that emits `Destroyed<Cell>`).

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct SiphonConfig {
    pub streak_window: f32,             // 2.0
    pub time_per_kill: f32,             // 0.5 — base reward unit (escalated by streak position)
}
```

## Components
```rust
#[derive(Resource, Debug, Default)]
pub(crate) struct SiphonStreak {
    pub window_remaining: f32,
    pub kill_count: u32,
}
```

Resource, not a component — tracks the global kill streak regardless of which bolt caused the kill.

## Messages
**Reads**: `Destroyed<Cell>` (from `rantzsoft_dmg`).
**Sends**: `IncreaseNodeTimer { delta: f32 }` (renamed from `ReverseTimePenalty` per TODO #3). Clamped by the consumer to `NodeTimer::total`.

## Systems

### `siphon_tick_streak`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Siphon)` + `in_state(NodeState::Playing)`.
- **Behavior**: If `window_remaining > 0.0`: `window_remaining -= delta_secs`. If drops to `<= 0.0`: reset streak (`kill_count = 0`, `window_remaining = 0`).
- **Ordering**: `.before(siphon_on_cell_destroyed)` — tick window first, then process new kills.

### `siphon_on_cell_destroyed`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyKill)`.
- **run_if**: `protocol_active(ProtocolKind::Siphon)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `Destroyed<Cell>` messages. For each:
  - If no active streak (`window_remaining <= 0.0`): start one — `kill_count = 1`, `window_remaining = streak_window`. No time added.
  - If active streak: `kill_count += 1`, `window_remaining = streak_window` (hard reset), emit `IncreaseNodeTimer { delta: config.time_per_kill * (kill_count - 1) as f32 }`. The `(N-1)` multiplier is the escalating reward.

### `siphon_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: `SiphonStreak::default()` — streaks do not persist across nodes.

## Pipeline position (dmg crate)

- **Trigger**: `Destroyed<Cell>` — from `rantzsoft_dmg` via `DeathPipelineSystems::ApplyKill`.
- **Emits**: `IncreaseNodeTimer` (state/run/node domain) — escalating reward.
- **Not a damage emitter or mutator**. Siphon does not participate in `MutateDamage` / `EmitDamage` / `EmitKill` sets.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` involvement.

## Cross-Domain Dependencies
- **cells / damage crate**: Reads `Destroyed<Cell>`. All kill sources funnel through the crate's death pipeline.
- **state/run/node**: Emits `IncreaseNodeTimer`. Consumer clamps to `NodeTimer::total`.
- **shared**: Reads `Time` for delta.

## Expected Behaviors (for test specs)

1. **First kill starts streak but adds no time** — `SiphonStreak::default()`, `Destroyed<Cell>`: `kill_count = 1`, `window_remaining = streak_window`. No `IncreaseNodeTimer` emitted.
2. **Second kill within window adds `time_per_kill * 1`** — `kill_count = 1`, `Destroyed<Cell>`: `kill_count = 2`, `IncreaseNodeTimer { delta: time_per_kill * 1 = 0.5 }` emitted.
3. **Third kill adds `time_per_kill * 2`** — `kill_count = 2`, `Destroyed<Cell>`: `kill_count = 3`, `IncreaseNodeTimer { delta: 1.0 }` emitted.
4. **Nth kill adds `time_per_kill * (N - 1)`** — escalating reward scales linearly with streak length.
5. **Window resets on each kill** — `window_remaining = 0.1`, `Destroyed<Cell>`: `window_remaining = streak_window` (hard reset, not additive).
6. **Streak expires after window elapses** — `window_remaining = 0.5`, `delta_secs = 0.6`: streak resets to `kill_count = 0`, `window_remaining = 0`.
7. **Kill after expired streak starts new streak (no time)** — expired streak + `Destroyed<Cell>`: new streak, first-kill rules apply.
8. **Multiple kills in same tick each escalate** — three `Destroyed<Cell>` in one tick, starting fresh: first starts streak (no time); second adds `time_per_kill * 1`; third adds `time_per_kill * 2`. Total time added: `time_per_kill * (1 + 2) = 3 * time_per_kill` across the tick.
9. **Streak cleared on node end** — `OnExit(Playing)`: `SiphonStreak::default()`.
10. **Added time clamped by consumer** — `IncreaseNodeTimer` consumer clamps the sum to `NodeTimer::total`. Siphon itself does not clamp.

## Edge Cases
- **AoE destroying many cells at once**: each destruction is a separate `Destroyed<Cell>`. Processed in order — first starts/continues streak, subsequent ones escalate.
- **Time overflow**: `IncreaseNodeTimer` consumer clamps. Siphon never overflows.
- **No cells left to kill**: streak naturally expires after `streak_window`.
- **Node timer expired**: `TimerExpired` already fired → node lost. Siphon's `IncreaseNodeTimer` has no effect after the state transition.
- **Interaction with Deadline**: Siphon extends the timer (reducing Deadline's window) but keeps you alive. Strategic tension, no mechanical conflict.

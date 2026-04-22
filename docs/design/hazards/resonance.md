# Hazard: Resonance

## Game Design

Every kill after the 2nd within a time window fires a slow-moving wave toward the breaker. Waves are dodgeable — slow travel speed, visually telegraphed. Punishes rapid kill chains (normally desirable). Player chooses between efficient play (fast kills, more waves) and cautious play (space out kills, fewer waves).

**Stacking formula**:
- Time window: `0.5s + 0.3s * (stack - 1)` — wider window = easier to trigger.
- Slow duration & strength: diminishing returns per level (logarithmic scaling prevents permafreeze).

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct ResonanceConfig {
    pub kills_to_trigger: u32,              // 2
    pub base_window: f32,                   // 0.5
    pub window_per_level: f32,              // 0.3
    pub wave_speed: f32,                    // world-units/sec
    pub wave_slow_base_duration: f32,       // seconds; effective = base * (1 + 0.2 * ln(stack))
    pub wave_slow_base_strength: f32,       // 0.0..0.95; effective = base * (1 + 0.15 * ln(stack))
    pub wave_max_lifetime: f32,             // seconds; orphaned waves auto-despawn
    pub wave_contact_threshold: f32,        // world-units; distance at which wave "touches" breaker
}
```

Effective slow per stack:
- `slow_duration = wave_slow_base_duration * (1.0 + 0.2 * ln(stack))`
- `slow_strength = wave_slow_base_strength * (1.0 + 0.15 * ln(stack))`

Populated from `HazardTuning::Resonance`.

## Components

```rust
#[derive(Component, Debug, Clone)]
pub(crate) struct ResonanceWave {
    pub target_pos: Vec2,          // snapshotted breaker position at spawn (dodgeable)
    pub speed: f32,                // world-units per second
    pub slow_duration: f32,        // seconds the slow persists on contact
    pub slow_strength: f32,        // 0.0..0.95; multiplier applied to breaker speed = 1.0 - strength
    pub age: f32,                  // seconds since spawn
    pub max_lifetime: f32,
    pub contact_threshold: f32,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct ResonanceTracker {
    /// Each entry: (timestamp of kill, world position of killed cell).
    /// Position is needed for wave spawn location when threshold hits.
    pub kills: Vec<(f32, Vec2)>,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct ResonanceActiveSlows {
    /// Per-source slow entries on the breaker. Keyed by unique source tag
    /// (one per wave). Removed when the slow's duration expires or on teardown.
    pub slows: HashMap<String, ResonanceSlowEntry>,
}

pub(crate) struct ResonanceSlowEntry {
    pub remaining: f32,
    pub strength: f32,
}
```

Slow entries are tracked in a `Resource` HashMap (`ResonanceActiveSlows`) rather than as per-breaker components. Rationale: cleanup is centralised (one drain + reverse loop on node teardown), and per-wave archetype churn on the breaker is avoided. Each wave's slow is uniquely source-tagged (`"hazard:resonance:wave:<entity_bits>"`) so multiple waves can coexist.

## Messages
**Reads**: `Destroyed<Cell>` (from `rantzsoft_dmg`) — used to append to `ResonanceTracker.kills`.
**Sends**: None directly. Spawns `ResonanceWave` entities; writes/removes entries in `ResonanceActiveSlows`; the breaker's movement system consults `ResonanceActiveSlows` during integration.

## Systems

### `resonance_track_kills`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyKill)`.
- **run_if**: `hazard_active(HazardKind::Resonance)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `CellDestroyed` (or `Destroyed<Cell>` + `Position2D` lookup). Appends `(current_time, position)` to `tracker.kills`. Prunes entries older than `current_time - effective_window`.

### `resonance_spawn_waves`
- **Schedule**: `FixedUpdate`, `.after(resonance_track_kills)`.
- **run_if**: `hazard_active(HazardKind::Resonance)` + `in_state(NodeState::Playing)`.
- **Behavior**: The spawner drains ONLY the EXCESS tail of `tracker.kills` beyond `kills_to_trigger`. Leading entries remain as "prior kills" carryover so the NEXT kill (after a wave fires) doesn't need to rebuild from zero; it only needs to reach `kills_to_trigger + 1` again. Post-spawn, `tracker.kills.len() == kills_to_trigger` is invariant (a `debug_assert!` in the impl pins it).
  For each drained `(timestamp, position)`, spawns a `ResonanceWave` entity at `position` with `target_pos` = breaker's current position at spawn time (snapshotted — dodgeable).

### `resonance_tick_waves`
- **Schedule**: `FixedUpdate`.
- **run_if**: `hazard_active(HazardKind::Resonance)` + `in_state(NodeState::Playing)`.
- **Behavior**: For each `ResonanceWave`:
  1. `age += delta_secs`. If `age > max_lifetime`, despawn (orphan auto-cleanup).
  2. Move toward `target_pos` at `speed * delta_secs`.
  3. If distance-to-breaker < `contact_threshold`: insert a `ResonanceSlowEntry { remaining: slow_duration, strength: slow_strength }` into `ResonanceActiveSlows.slows` keyed by `"hazard:resonance:wave:<entity_bits>"`. Despawn the wave.

### `resonance_tick_slows`
- **Schedule**: `FixedUpdate`, `.before(BreakerSystems::IntegrateMotion)`.
- **run_if**: `hazard_active(HazardKind::Resonance)` + `in_state(NodeState::Playing)`.
- **Behavior**: Decrements each `ResonanceSlowEntry.remaining -= delta_secs`. Removes entries at `remaining <= 0.0`. Breaker movement reads the aggregate slow (1.0 - max_strength or product, per breaker-domain choice) during integration.

## Pipeline position (dmg crate)

- **Trigger**: reads `Destroyed<Cell>` from the `rantzsoft_dmg` crate — each cell death appends `(timestamp, position)` to `ResonanceTracker.kills`.
- **Not a damage emitter or mutator.** Resonance does NOT participate in any `DeathPipelineSystems` set. It spawns wave entities (its own `ResonanceWave` component, not related to the damage chain) and applies speed slows to the breaker via the centrally-tracked `ResonanceActiveSlows` resource.
- **Ordering**: `resonance_track_kills` runs `.after(DeathPipelineSystems::ApplyKill)`; `resonance_spawn_waves` / `resonance_tick_waves` / `resonance_tick_slows` run in `FixedUpdate` without further chain dependencies.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` involvement.

## Stacking Behavior

| Stack | Window | Meaning |
|-------|--------|---------|
| 1 | 0.5s | Need 3+ kills in 0.5s to trigger |
| 2 | 0.8s | Wider window — easier to trigger |
| 3 | 1.1s | Most multi-hit combos trigger |
| 5 | 1.7s | Nearly any consecutive pair within 2s triggers |

Window expansion is the primary scaling lever. Logarithmic slow scaling prevents the breaker from being permanently frozen at high stacks.

## Cross-Domain Dependencies
- **cells**: Reads cell world position (via `CellDestroyed` companion or `Position2D` lookup).
- **damage crate (`rantzsoft_dmg`)**: Reads `Destroyed<Cell>`.
- **breaker**: Reads `ResonanceActiveSlows` aggregate during integration (breaker-domain query).
- **fx**: Reads `ResonanceWave` for visual telegraphing (out of scope here).

## Expected Behaviors (for test specs)

1. **No wave on 1st or 2nd kill within window at stack 1** — window 0.5s, `kills_to_trigger: 2`: 2 kills appended; no `ResonanceWave` spawned.
2. **Wave spawns on 3rd kill within window at stack 1** — 2 entries already present, 3rd kill arrives: 1 `ResonanceWave` entity spawned; `tracker.kills.len() == 2` post-spawn (invariant).
3. **Window expands with stack 3** — window 1.1s: 3 kills spread across 1.1s (but not within 0.5s) trigger a wave.
4. **Wave travels toward breaker and applies slow** — wave at (100,300), breaker at (100,50), `speed: 200`: ~1.25s to reach; on contact, inserts `ResonanceSlowEntry` into `ResonanceActiveSlows`; despawns wave.
5. **Kills outside window don't trigger** — stack 1, last kill 0.6s ago: old entry pruned; count 1; no wave.
6. **Orphaned wave auto-despawns at max_lifetime** — wave age > `max_lifetime`: despawned even if it never reached the breaker.
7. **Multiple waves coexist with unique source keys** — 2 waves contact breaker on same tick: 2 distinct entries in `ResonanceActiveSlows.slows` keyed by `"hazard:resonance:wave:<entity_bits>"`.

## Edge Cases
- **Resonance + Echo Cells**: ghost kills also count. A cluster of ghosts cleared rapidly can burst-trigger waves. Intentional.
- **Multiple waves in flight**: each applies its slow independently; breaker-domain aggregation combines them (product or max — breaker's choice).
- **Breaker dodge**: `target_pos` is snapshotted at wave spawn. Player can dodge by moving. Never tracks — the dodgeability is the design.
- **Wave lifetime cap**: `max_lifetime` prevents orphans (breaker moves away, wave never reaches). Despawn on age > cap.
- **Kill-timestamp precision**: uses fixed-timestep game time, not wall clock — deterministic replays.
- **Rapid kills same frame**: multiple cells die same tick (e.g., Tether chain); all share the same timestamp. Counts as simultaneous within the window.
- **Cleanup**: `ResonanceTracker` + `ResonanceActiveSlows` cleared at node end or run end. `ResonanceWave` entities despawned via cleanup markers. `ResonanceConfig` removed at run end.

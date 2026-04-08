# Hazard: Resonance

## Game Design

Every kill after the 2nd within a time window fires a slow-moving wave toward the breaker. Waves are dodgeable -- they have slow travel speed and are visually telegraphed. This punishes rapid kill chains (which are normally desirable). The player must choose between efficient play (fast kills, more waves) and cautious play (space out kills, fewer waves).

**Stacking formula**:
- Time window: `0.5s + 0.3s * (stack - 1)` (wider window = easier to trigger)
- Slow duration and strength have diminishing returns per level

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct ResonanceConfig {
    pub kills_to_trigger: u32,  // 2 (kills within window before waves start)
    pub base_window: f32,       // 0.5 seconds
    pub window_per_level: f32,  // 0.3 seconds per additional stack
    pub wave_speed: f32,        // travel speed toward breaker (units/sec, tuning TBD)
    pub wave_slow_duration: f32, // how long the slow effect lasts on breaker (tuning TBD)
    pub wave_slow_strength: f32, // slow multiplier (e.g., 0.5 = 50% speed, tuning TBD)
}
```

**Diminishing returns on slow**: `effective_slow_duration = base_duration * (1.0 + 0.2 * ln(stack))` and `effective_slow_strength = base_strength * (1.0 + 0.15 * ln(stack))`. Logarithmic scaling prevents the breaker from being permanently frozen at high stacks.

## Components

```rust
/// Tracks recent kills within the resonance time window.
/// Managed as a per-run resource (not per-entity -- resonance is global).
#[derive(Resource, Debug)]
pub(crate) struct ResonanceTracker {
    /// Timestamps of recent kills (kept pruned to the current window).
    pub kill_timestamps: Vec<f32>,
}

/// A resonance wave entity traveling toward the breaker.
#[derive(Component, Debug)]
pub(crate) struct ResonanceWave {
    pub speed: f32,
    pub slow_duration: f32,
    pub slow_strength: f32,
}
```

`ResonanceTracker` is a resource (not a component) because resonance tracks global kill rate, not per-bolt or per-cell kills.

## Messages

**Reads**: `CellDestroyedAt` (to track kill timestamps)
**Sends**: Spawns wave entities (own domain). When a wave reaches the breaker, it may send a slow effect message (TBD -- depends on whether breaker slow is an effect or a direct component).

## Systems

1. **`resonance_track_kills`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Resonance)` AND `in_state(NodeState::Playing)`
   - Ordering: After cell death processing
   - Behavior:
     1. Read `CellDestroyedAt` messages, record current game time for each
     2. Compute effective window: `base_window + window_per_level * (stack - 1)`
     3. Prune timestamps older than `current_time - effective_window`
     4. After pruning, if `kill_timestamps.len() > kills_to_trigger`, fire a wave for each kill beyond the threshold
     5. Spawn wave entity at the destroyed cell's position, aimed at the breaker

2. **`resonance_wave_travel`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Resonance)` AND `in_state(NodeState::Playing)`
   - Behavior:
     1. For each `ResonanceWave` entity, move it toward the breaker position at `wave.speed` units/sec
     2. If wave reaches the breaker (distance < threshold), apply slow effect and despawn wave

3. **`resonance_wave_apply_slow`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Resonance)` AND `in_state(NodeState::Playing)`
   - Ordering: After `resonance_wave_travel`
   - Behavior:
     1. When a wave reaches the breaker, apply a movement slow (reduced breaker speed for `slow_duration` seconds at `slow_strength` multiplier)
     2. Implementation: either send a message to the breaker domain or apply a `BreakerSlow` component directly. Prefer message-driven approach.
     3. Despawn the wave entity

## Stacking Behavior

| Stack | Time window | Effective meaning |
|-------|-------------|-------------------|
| 1     | 0.5s        | Must kill 3+ cells in 0.5s to trigger waves |
| 2     | 0.8s        | Wider window -- easier to trigger |
| 3     | 1.1s        | Most multi-hit combos now trigger waves |
| 5     | 1.7s        | Nearly any consecutive pair of kills within 2 seconds triggers |

The window expansion is the primary scaling mechanism. The slow effect's diminishing returns prevent hard-locking the breaker.

## Cross-Domain Dependencies

| Domain | Interaction | Message |
|--------|------------|---------|
| `cells` | Reads cell destruction events | `CellDestroyedAt` message (read) |
| `breaker` | Applies slow effect when wave reaches breaker | Message TBD (slow effect or `ApplyBreakerSlow`) |

**Wave entities**: Owned by the hazard domain. The `fx` domain reads `ResonanceWave` components for visual rendering (expanding ring, color pulse, etc.). FX details are out of scope.

## Expected Behaviors (for test specs)

1. **No wave on 1st or 2nd kill within window at stack=1**
   - Given: Stack=1, window=0.5s, kills_to_trigger=2
   - When: 1st and 2nd cells destroyed within 0.5s
   - Then: No `ResonanceWave` spawned (threshold not exceeded)

2. **Wave spawns on 3rd kill within window at stack=1**
   - Given: Stack=1, 2 kills already in `kill_timestamps` within 0.5s
   - When: 3rd cell destroyed within the window
   - Then: `ResonanceWave` entity spawned at destroyed cell's position

3. **Window expands with stack count at stack=3**
   - Given: Stack=3, window=1.1s
   - When: 3 cells destroyed within 1.1s (but not within 0.5s)
   - Then: Wave spawns (would not have triggered at stack=1)

4. **Wave travels toward breaker and applies slow**
   - Given: `ResonanceWave` at (100.0, 300.0), breaker at (100.0, 50.0), wave_speed=200.0
   - When: ~1.25 seconds pass
   - Then: Wave reaches breaker, slow effect applied, wave despawned

5. **Kills outside window do not trigger waves**
   - Given: Stack=1, window=0.5s, last kill 0.6s ago
   - When: New cell destroyed
   - Then: Old timestamp pruned, kill count=1, no wave (threshold not exceeded)

6. **System does not run when hazard is inactive**
   - Given: Resonance not in `ActiveHazards`
   - When: Cells destroyed rapidly
   - Then: No `ResonanceTracker` updates, no waves spawned

## Edge Cases

- **Resonance + Echo Cells synergy**: Destroying echo ghosts counts as kills. A cluster of ghosts being cleared rapidly can trigger a burst of resonance waves. This is intentional.
- **Multiple waves in flight**: Several waves can be traveling toward the breaker simultaneously. Each applies its slow independently. Multiple slows should stack (compound duration or increase strength) -- the exact stacking model for breaker slow needs design decision.
- **Breaker dodge**: Waves travel slowly and are aimed at the breaker's position at spawn time (or track the breaker -- design decision). If aimed at spawn position, the player can dodge by moving. If tracking, the player must time a dash. Recommend: aimed at spawn position (dodgeable without dash, dash makes it trivial).
- **Wave lifetime cap**: Waves should have a maximum lifetime (e.g., 10s) to prevent orphaned waves that never reach the breaker (if the breaker moves away). Despawn on lifetime expiry.
- **Kill timestamp precision**: Use the game's fixed timestep time, not wall clock. This ensures deterministic behavior in replays.
- **Rapid kills in one frame**: If multiple cells die in the same frame (e.g., chain reaction from Tether), they all have the same timestamp. This counts as simultaneous kills within the window.
- **Cleanup**: `ResonanceTracker` resource cleared at node end or run end. `ResonanceWave` entities despawned at node end. `ResonanceConfig` removed at run end.

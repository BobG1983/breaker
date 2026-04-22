# Resonance — full component shapes + kills-beyond-threshold semantics in the canonical design doc

## Target file

`docs/design/hazards/resonance.md` (promoted during this sweep).

## What the target doc must say

§ Components — full impl shapes (not abbreviated):

```rust
#[derive(Component, Debug, Clone)]
pub(crate) struct ResonanceWave {
    pub target_pos: Vec2,          // snapshotted breaker position at spawn (dodgeable)
    pub speed: f32,                // world-units per second
    pub slow_duration: f32,        // seconds the slow persists on contact
    pub slow_strength: f32,        // 0.0..0.95; multiplier applied to breaker speed = 1.0 - strength
    pub age: f32,                  // seconds since spawn; used for lifetime cap
    pub max_lifetime: f32,         // seconds; orphaned waves auto-despawn at this age
    pub contact_threshold: f32,    // world-unit distance at which the wave touches the breaker
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

Under § Components, add a paragraph:

> Slow entries are tracked in a `Resource` HashMap (`ResonanceActiveSlows`) rather than as per-breaker components. Rationale: cleanup is centralised (one drain + reverse loop on node teardown), and per-wave archetype churn on the breaker is avoided. Each wave's slow is uniquely source-tagged (`"hazard:resonance:wave:<entity_bits>"`) so multiple waves can coexist.

§ Systems — for `resonance_spawn_waves`, describe the drain:

> The spawner drains ONLY the EXCESS tail of `tracker.kills` beyond `kills_to_trigger`. Leading entries remain as "prior kills" carryover so the NEXT kill (after a wave fires) doesn't need to rebuild from zero; it only needs to reach `kills_to_trigger + 1` again. Post-spawn, `tracker.kills.len() == kills_to_trigger` is invariant (a `debug_assert!` in the impl pins it).

## Pipeline position (dmg crate)

- **Trigger**: reads `Destroyed<Cell>` from the `rantzsoft_dmg` crate — each cell death appends `(timestamp, position)` to `ResonanceTracker.kills`.
- **Not a damage emitter or mutator.** Resonance does NOT participate in any `DeathPipelineSystems` set. It spawns wave entities (its own `ResonanceWave` component, not related to the damage chain) and applies speed slows to the breaker via the centrally-tracked `ResonanceActiveSlows` resource.
- **Ordering**: `resonance_track_kills` runs `.after(DeathPipelineSystems::ApplyKill)`; `resonance_spawn_waves` / `resonance_tick_waves` / `resonance_tick_slows` run in `FixedUpdate` without further chain dependencies.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` involvement.

## Why

The component shapes and drain semantics are load-bearing for impl correctness and for reading the mechanic. The doc must match what the code does.

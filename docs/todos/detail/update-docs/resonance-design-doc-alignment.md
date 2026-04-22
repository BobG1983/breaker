# Resonance: align design doc with impl (richer Components, kills-beyond-threshold semantics)

## Problems addressed

- `audit/hazards/resonance.md` Issue 3 \u2014 Design \u00a7Components for `ResonanceWave` incomplete; impl adds `target_pos`, `age`, `max_lifetime`, `contact_threshold`.
- `audit/hazards/resonance.md` Issue 4 \u2014 Design \u00a7Components for `ResonanceTracker` says `Vec<f32>`; impl uses `Vec<(f32, Vec2)>` (needs position for wave spawn location).
- `audit/hazards/resonance.md` Issue 5 \u2014 `ResonanceActiveSlows` and `ResonanceSlowEntry` not in design \u00a7Components.
- `audit/hazards/resonance.md` Issue 6 \u2014 Resource-based slow tracking (not per-entity) is an architectural choice not mentioned in design.
- `audit/hazards/resonance.md` Issue 7 \u2014 "Kills-beyond-threshold" drain semantics undocumented in design.

## Remediation

Open `docs/todos/detail/mod-system-design/hazards/resonance.md`.

\u00a7Components \u2014 replace the Wave and Tracker definitions with full impl shapes:

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

Add under \u00a7Components a new paragraph:

> Slow entries are tracked in a `Resource` HashMap (`ResonanceActiveSlows`) rather than as per-breaker components. Rationale: cleanup is centralized (one drain + reverse loop on node teardown), and per-wave archetype churn on the breaker is avoided. Each wave's slow is uniquely source-tagged (`"hazard:resonance:wave:<entity_bits>"`) so multiple waves can coexist.

\u00a7Systems `resonance_spawn_waves` step \u2014 describe the kills-beyond-threshold drain:

> The spawner drains ONLY the EXCESS tail of `tracker.kills` beyond `kills_to_trigger`. Leading entries remain as "prior kills" carryover so the NEXT kill (after a wave fires) doesn't need to rebuild from zero; it only needs to reach `kills_to_trigger + 1` again. Post-spawn, `tracker.kills.len() == kills_to_trigger` is invariant (add a debug_assert in the impl to pin this).

No code change. No test change. Doc-only alignment.

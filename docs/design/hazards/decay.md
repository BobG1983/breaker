# Hazard: Decay

## Game Design

Node timer ticks faster. 15% base + 5% per stack. Stacking makes the timer visibly melt away, creating urgency that compounds with every other hazard (less time to deal with ghosts, regen, fracture debris, etc.).

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct DecayConfig {
    pub base_percent: f32,          // 15.0
    pub per_level_percent: f32,     // 5.0
}
```

Populated from `HazardTuning::Decay`.

## Components
None.

## Messages
**Reads**: `Time` for delta, `Option<Res<ActiveHazards>>` for stack count (harness-safety: `Option<Res>` so the system is usable in tests without the full `HazardPlugin` installed — per TODO #9).
**Sends**: `ReduceNodeTimer { delta: f32 }` (renamed from `ApplyTimePenalty` per TODO #4 — field `seconds` renamed to `delta`). Owned by `state/run/node` domain.

## Systems

### `decay_tick`
- **Schedule**: `FixedUpdate`, `.after(NodeSystems::TickTimer)`.
- **run_if**: `hazard_active(HazardKind::Decay)` + `in_state(NodeState::Playing)`.
- **Behavior**: Computes extra drain: `extra = delta_secs * (base_percent + per_level_percent * (stack - 1)) / 100.0`. Emits `ReduceNodeTimer { delta: extra }`.
- **Harness-safety**: Accepts `Option<Res<ActiveHazards>>`. If the resource is absent (e.g., in a unit-test harness without `HazardPlugin`), the system returns without emitting.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Decay does not participate in any `DeathPipelineSystems` set.
- **Trigger**: FixedUpdate tick.
- **Emits**: `ReduceNodeTimer` — node domain drains the timer.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement.

## Stacking Behavior

Linear: `speedup_percent = base_percent + per_level_percent * (stack - 1)`

| Stack | Speedup | Effective drain |
|-------|---------|-----------------|
| 1 | 15% | 1.15× normal |
| 3 | 25% | 1.25× normal |
| 10 | 60% | 1.60× normal |

No cap in formula — infinite play is meant to become impossible eventually.

## Cross-Domain Dependencies
- **state/run/node**: Emits `ReduceNodeTimer`. Consumer drains `NodeTimer.remaining`.
- **shared**: Reads `Time` for delta.

## Expected Behaviors (for test specs)

1. **Timer drains faster at stack 1** — `base_percent: 15.0`, `delta_secs: 0.1`, stack 1: emits `ReduceNodeTimer { delta: 0.015 }`.
2. **Timer drains faster at stack 3** — stack 3: `delta_secs: 0.1`: emits `ReduceNodeTimer { delta: 0.025 }`.
3. **System does not run when Decay is inactive** — `hazard_active(Decay) = false`: system skipped.
4. **System does not run outside `NodeState::Playing`** — paused state: system skipped.
5. **Zero delta produces zero penalty** — `delta_secs = 0.0`: emits `ReduceNodeTimer { delta: 0.0 }` (or no message; implementation choice).
6. **Harness-safety with absent `ActiveHazards`** — `Option<Res<ActiveHazards>>` is `None`: system returns without emitting.

## Edge Cases
- **Decay + Renewal**: two clocks — faster drain + regening cells. Emergent pressure, no special code.
- **Cleanup**: `DecayConfig` removed at run end.
- **Very short nodes**: drain is proportional to real time, not remaining time. Noticeable but not catastrophic.
- **No max cap in formula**: add a `max_percent` field later if infinite-scaling becomes undesirable.

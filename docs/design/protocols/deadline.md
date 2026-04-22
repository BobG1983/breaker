# Protocol: Deadline

## Category
`code-driven` (per TODO #7 — previously `effect-tree`; now dispatched from code via the stamp dispatcher + fire/reverse commands).

## Game Design
You WANT to slow-play 75% of the node, then explode in the danger zone.

When the node timer drops below 25%, all bolts get 2x speed + 2x damage until the node ends.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct DeadlineConfig {
    pub threshold_fraction: f32,    // 0.25
    pub speed_multiplier: f32,      // 2.0
    pub damage_multiplier: f32,     // 2.0
}
```

Populated from `ProtocolTuning::Deadline`. RON carries tuning only — no effect tree.

## Components
None owned by Deadline. The timer-threshold trigger is a code-driven condition poll. Effects land on bolts via `commands.fire_effect` → `EffectStack<SpeedBoostConfig>` on each bolt (+ `DamageBoostStack` persistent entry on each bolt).

## Messages
**Reads**: `NodeTimerTick` / `NodeTimer` resource (run/node domain) to detect threshold crossing.
**Sends**: None directly. Effects dispatched through `commands.fire_effect` (source `"protocol:deadline"`) on each bolt when threshold crosses. Effects reversed via `commands.reverse_effect` on node end.

## Systems

### `deadline_check_threshold`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Deadline)` + `in_state(NodeState::Playing)`.
- **Behavior**: Polls `NodeTimer`. If `remaining / total <= threshold_fraction` AND the threshold has NOT yet fired this node: for each active bolt, fires `SpeedBoost(speed_multiplier)` and `DamageBoost(damage_multiplier)` effects with source `"protocol:deadline"`. Marks the threshold fired (per-node flag, cleared on `OnEnter(NodeState::Playing)`). Registers with `SpawnStampRegistry` so bolts spawned after threshold (e.g., from Fission) also receive the effects.
- **Ordering**: After the node-timer tick system.

### `deadline_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: Reverses all `"protocol:deadline"`-sourced effects via `commands.reverse_all_by_source(ctx.world, "protocol:deadline")`. Clears the threshold-fired flag for the next node.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Deadline does not participate in any `DeathPipelineSystems` set.
- **Trigger**: node-timer threshold crossing (condition poll, not a message).
- **Writes**: `DamageBoostStack::persistent` entries on bolts (source-tagged `"protocol:deadline"`), aggregated by `DeathPipelineSystems::ApplyDamageBoosts` on every `DamageDealt<Cell>`. Plus `EffectStack<SpeedBoostConfig>` entries on bolts for the speed side.
- **No** direct `DamageDealt<T>` emission.

## Cross-Domain Dependencies
- **run/node**: Reads `NodeTimer`. Reads `NodeState` transitions.
- **bolt**: Writes `DamageBoostStack::persistent` + `EffectStack<SpeedBoostConfig>` entries on all active bolts.
- **effect_v3**: Uses `fire_effect` / `reverse_effect` / `reverse_all_by_source` + `SpawnStampRegistry` for retroactive spawn handling.
- **damage crate (`rantzsoft_dmg`)**: `ApplyDamageBoosts` consumes the persistent entry on every damage emission.

## Expected Behaviors (for test specs)

1. **Bolt speed doubles when timer crosses 25%** — `NodeTimer.remaining / total = 0.24`, `speed_multiplier = 2.0`, bolt speed 400: after `deadline_check_threshold`, bolt speed 800 (via `EffectStack<SpeedBoostConfig>` aggregate).
2. **Bolt damage doubles when timer crosses 25%** — `damage_multiplier = 2.0`, bolt base damage 10: after threshold, `DamageDealt<Cell>` emits 20 (via `ApplyDamageBoosts` aggregating the persistent entry).
3. **Effects stack multiplicatively with chip effects** — existing chip `DamageBoost(1.5)` + Deadline `DamageBoost(2.0)`: effective `10 * 1.5 * 2.0 = 30`.
4. **Effects persist until node end** — threshold crosses at 25%; effects stay until `OnExit(NodeState::Playing)` reverses all `"protocol:deadline"` entries.
5. **Threshold fires only once per node** — timer continues to 20%, 15%, etc.: no additional fires (per-node flag).
6. **No effect if timer never reaches threshold** — node ends with timer above 25%: no fires, no reverses needed.
7. **Multiple bolts all receive the boost** — 3 bolts in play when threshold crosses: each gets its own source-tagged entries.
8. **Bolt spawned after threshold receives effects** — Fission spawns a new bolt after threshold; `SpawnStampRegistry` applies Deadline effects on spawn.

## Edge Cases
- **Node starts below threshold**: fires immediately at `OnEnter(NodeState::Playing)` on the first tick.
- **Bolt-lost during Deadline**: normal bolt-lost fires; remaining bolts keep their effects. Lost bolt's entries die with the entity.
- **Node-end cleanup**: `reverse_all_by_source("protocol:deadline")` removes all traces so the next node starts clean.
- **Interaction with Kickstart**: both can be active; they rarely overlap (Kickstart in first 3s; Deadline below 25%). If they do, effects stack multiplicatively.

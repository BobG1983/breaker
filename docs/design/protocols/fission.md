# Protocol: Fission

## Category
`custom-system`

## Game Design
You WANT to maximize destruction volume for bolt splits.

- Every Nth cell destroyed (default 8) splits one bolt into two.
- New bolt inherits parent's effects.
- The split bolts persist — they survive through node ends for the rest of the run.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct FissionConfig {
    pub kills_per_split: u32,           // 8
    pub divergence_angle_rad: f32,      // ~15° (tunable RON)
}
```

## Components
```rust
/// Per-node kill counter. Resets at node boundary so the split cadence restarts each node.
#[derive(Resource, Debug, Default)]
pub(crate) struct FissionCounter {
    pub kills: u32,
}
```

`FissionCounter` is per-node state — cleared in `OnEnter(NodeState::Playing)`. This differs from older designs where the counter persisted across nodes: per-node reset gives more predictable split cadence and avoids carry-over from a boss-tier node into a subsequent node.

## Messages
**Reads**: `Destroyed<Cell>` (from `rantzsoft_dmg`).
**Sends**: None directly. Spawns a new bolt via `Bolt::builder().replicate_of(primary).rendered().spawn(...)` (TODO #8 builder API). The `.replicate_of(primary)` method clones the primary bolt's relevant state (position, velocity, `BoundEffects`, `StagedEffects`). The `.rendered()` step attaches visuals.

## Systems

### `fission_on_cell_destroyed`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyKill)` (so `Destroyed<Cell>` is populated).
- **run_if**: `protocol_active(ProtocolKind::Fission)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `Destroyed<Cell>` messages. For each kill, `FissionCounter.kills += 1`. If `kills >= config.kills_per_split`:
  1. Resets `kills = 0`.
  2. Selects the primary bolt (`Query<Entity, With<PrimaryBolt>>`).
  3. Spawns a new bolt via `Bolt::builder().replicate_of(primary).rendered().spawn(...)`. The new bolt:
     - Inherits position from primary.
     - Inherits velocity magnitude; direction is rotated by `config.divergence_angle_rad`.
     - Inherits `BoundEffects` + `StagedEffects` (cloned).
     - Receives `ExtraBolt` marker (only the original primary keeps `PrimaryBolt`).

### `fission_reset_counter_on_node_enter`
- **Schedule**: `OnEnter(NodeState::Playing)`.
- **Behavior**: `FissionCounter.kills = 0`.

## Pipeline position (dmg crate)

- **Trigger**: reads `Destroyed<Cell>` — from `rantzsoft_dmg` via `DeathPipelineSystems::ApplyKill`.
- **Not a damage emitter or mutator**. Fission spawns bolt entities; it does not emit `DamageDealt<T>` or `HealDealt<T>`.
- **Ordering**: `.after(DeathPipelineSystems::ApplyKill)` — destruction messages must be populated.
- **No** `DamageBoostStack` / `VulnerableStack` involvement.

## Cross-Domain Dependencies
- **cells / damage crate**: Reads `Destroyed<Cell>`.
- **bolt**: Uses `Bolt::builder().replicate_of(primary).rendered().spawn(...)`. Reads `PrimaryBolt`. Writes `ExtraBolt` on new bolts.
- **effect_v3**: The new bolt's `BoundEffects` / `StagedEffects` clone ensures existing chip effects carry over.

## Expected Behaviors (for test specs)

1. **Kill counter increments on cell destruction** — `kills = 0`, `Destroyed<Cell>`: `kills = 1`.
2. **Nth kill triggers a split** — `kills = 7`, `kills_per_split = 8`, `Destroyed<Cell>`: `kills = 0`; new bolt spawned.
3. **New bolt spawns at primary position** — primary at (200, 300) velocity (150, 400): new bolt at (200, 300), velocity magnitude 425.7, direction rotated by `divergence_angle_rad`.
4. **New bolt inherits primary's effects** — primary has `BoundEffects(X)` + `StagedEffects(Y)`: new bolt has cloned copies.
5. **New bolt marked ExtraBolt** — only the original primary retains `PrimaryBolt`; the fissioned bolt gets `ExtraBolt`.
6. **Counter resets at node boundary** — `kills = 5` at `OnExit(Playing)`, next `OnEnter(Playing)`: `kills = 0`.
7. **Multiple destructions in one tick** — 3 `Destroyed<Cell>` in same tick, `kills_per_split = 2`: first triggers split (kills 1→2→reset→0), second+third accumulate (kills 1, 2→reset→0 if ordering allows; concretely: fission triggers at the tick where `kills` reaches threshold).
8. **Persistence across nodes** — fissioned bolt exists after `OnExit(Playing)` → `OnEnter(Playing)`; only `FissionCounter` resets.

## Edge Cases
- **All kill sources count**: bolt impact, shockwave, explode, chain lightning — any `Destroyed<Cell>` increments.
- **No active primary**: if the primary is lost before a split triggers (all bolts gone), spawn is skipped — `Bolt::builder().replicate_of(None)` returns without spawning.
- **Divergence angle is fixed per split**: alternating direction or stochastic angles are a potential future tuning knob.
- **Interaction with Conductor**: new bolt spawns as `ExtraBolt`. Perfect-bumping it promotes to primary (effects swap).
- **Bolt cap**: no explicit cap on bolt count from Fission. Over a long run, bolt count can grow.
- **Parent bolt velocity unchanged** by the split — only the new bolt gets the rotated velocity.

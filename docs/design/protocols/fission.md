# Protocol: Fission

## Category
`custom-system`

## Game Design
You WANT to maximize destruction volume for bolt splits.

- Every Nth cell destroyed (default 10) splits one bolt into two.
- New bolt inherits parent's effects.
- The split bolts persist — they survive through node ends for the rest of the run.

## Config Resource
```rust
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct FissionConfig {
    pub kills_per_split:      u32,           // 10
    pub divergence_angle_rad: f32,           // ~15° (0.261799 rad, tunable RON)
}
```

## Components
```rust
/// Persistent-across-nodes cell-kill tracker.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FissionCounter {
    pub kills: u32,
}
```

`FissionCounter` persists across nodes — it is NOT reset on node boundary. The counter is initialised by `ProtocolPlugin::build` (mirrors Siphon precedent) and removed by `fission_cleanup_run` on `OnExit(MenuState::Main)` (run start). Split cadence therefore accumulates across nodes for the duration of the run.

## Messages
**Reads**: `Destroyed<Cell>` (from `rantzsoft_dmg`).
**Sends**: None directly. Spawns a new bolt via `Bolt::builder().at_position(..).definition(..).with_velocity(..).extra().headless().spawn(&mut commands)`. Parent bolt's `BoundEffects` / `StagedEffects` are cloned onto the new bolt via direct `commands.entity(new_bolt).insert(..)` calls after spawn.

## Systems

### `fission_on_cell_destroyed`
- **Schedule**: `FixedUpdate`, `.after(DmgSystems::ApplyKill)` (so `Destroyed<Cell>` is populated).
- **run_if**: in-body gate via `ProtocolGate::is_closed_for(ProtocolKind::Fission)` — drains reader and returns when inactive.
- **Behavior**: Reads `Destroyed<Cell>` messages. For each kill, `FissionCounter.kills += 1 (saturating)`. If `kills < config.kills_per_split`, continue. At threshold:
  1. Resets `kills = 0`.
  2. Picks the parent bolt: `msg.killer` if it resolves to a live bolt in `FissionBoltQuery` (all live bolts `With<Bolt>, Without<Dead>`), else the first bolt the query yields. If no bolt alive, skips the spawn (counter already reset).
  3. Looks up the parent's `BoltDefinition` in `BoltRegistry` via `BoltDefinitionRef.0`.
  4. Rotates parent velocity clockwise by `config.divergence_angle_rad` via `Velocity2D::rotate_by`.
  5. Spawns new bolt: `.extra().headless()`.
  6. Clones parent's `BoundEffects` / `StagedEffects` onto the new bolt via direct `commands.entity(new_bolt).insert(..)`.

### `fission_cleanup_run`
- **Schedule**: `OnExit(MenuState::Main)`.
- **Behavior**: Removes both `FissionConfig` and `FissionCounter` resources. Harness-safe (`remove_resource` on absent resource is a no-op).

## Pipeline position (dmg crate)

- **Trigger**: reads `Destroyed<Cell>` — from `rantzsoft_dmg` via `DmgSystems::ApplyKill`.
- **Not a damage emitter or mutator**. Fission spawns bolt entities; it does not emit `DamageDealt<T>` or `HealDealt<T>`.
- **Ordering**: `.after(DmgSystems::ApplyKill)` — destruction messages must be populated.
- **No** `DamageBoostStack` / `VulnerableStack` involvement.

## Cross-Domain Dependencies
- **cells / damage crate**: Reads `Destroyed<Cell>`.
- **bolt**: Uses `Bolt::builder().extra().headless().spawn(...)`. Reads `BoltDefinitionRef` + `BoltRegistry` to look up the parent definition. Reads `Velocity2D` + `Position2D` for split parameters. Writes `ExtraBolt` marker on new bolts (via `.extra()`).
- **effect_v3**: Clones parent `BoundEffects` / `StagedEffects` onto the new bolt so existing chip effects carry over.

## Expected Behaviors (for test specs)

1. **Kill counter increments on cell destruction** — `kills = 0`, `Destroyed<Cell>`: `kills = 1`.
2. **Nth kill triggers a split** — `kills = 9`, `kills_per_split = 10`, `Destroyed<Cell>`: `kills = 0`; new bolt spawned.
3. **New bolt spawns at primary position** — primary at (200, 300) velocity (150, 400): new bolt at (200, 300), velocity magnitude 425.7, direction rotated by `divergence_angle_rad`.
4. **New bolt inherits primary's effects** — primary has `BoundEffects(X)` + `StagedEffects(Y)`: new bolt has cloned copies.
5. **New bolt marked ExtraBolt** — only the original primary retains `PrimaryBolt`; the fissioned bolt gets `ExtraBolt`.
6. **Counter persists across node boundaries** — `kills = 5` at `OnExit(Playing)`, next `OnEnter(Playing)`: `kills = 5` (counter carries over; only removed at run start via `OnExit(MenuState::Main)`).
7. **Multiple destructions in one tick** — 3 `Destroyed<Cell>` in same tick, `kills_per_split = 2`: first triggers split (kills 1→2→reset→0), second+third accumulate (kills 1, 2→reset→0 if ordering allows; concretely: fission triggers at the tick where `kills` reaches threshold).
8. **Persistence across nodes** — fissioned bolt exists after `OnExit(Playing)` → `OnEnter(Playing)`; `FissionCounter` also persists (does not reset at node boundary).

## Edge Cases
- **All kill sources count**: bolt impact, shockwave, explode, chain lightning — any `Destroyed<Cell>` increments.
- **No active bolt**: if no live bolt is found before a split triggers, spawn is skipped — counter already reset to 0.
- **Divergence angle is fixed per split**: alternating direction or stochastic angles are a potential future tuning knob.
- **Interaction with Conductor**: new bolt spawns as `ExtraBolt`. Perfect-bumping it promotes to primary (effects swap).
- **Bolt cap**: no explicit cap on bolt count from Fission. Over a long run, bolt count can grow.
- **Parent bolt velocity unchanged** by the split — only the new bolt gets the rotated velocity.

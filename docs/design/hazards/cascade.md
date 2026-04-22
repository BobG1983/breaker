# Hazard: Cascade

## Game Design

Destroyed cells heal nearby cells. +10 HP base, +5 HP per stacked level. Creates a "whack-a-mole" dynamic: killing a cell makes its neighbors tougher. Player must plan kill order — work edges inward to minimize how many neighbors benefit. At high stacks, middle-of-cluster kills heal massively, potentially undoing recent damage.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct CascadeConfig {
    pub base_heal: f32,         // 10.0
    pub heal_per_level: f32,    // 5.0
    pub neighbor_radius: f32,   // e.g., 70.0 — tuned to catch orthogonal grid neighbors at standard spacing
}
```

Populated from `HazardTuning::Cascade`.

## Components
None. Cascade is a reactive hazard — reads cell destruction and emits heal messages.

## Messages
**Reads**: `CellDestroyed { entity, position }` (cells-domain companion message that snapshots the victim's world position before `ProcessDespawn` — see `rantzsoft_dmg::Destroyed<Cell>` upstream trigger).
**Sends**: `HealDealt<Cell> { healer, target, amount, cap: HealCap::Starting, source, _marker }` from `rantzsoft_dmg`. One per in-range living cell. Source: `"hazard:cascade"`.

## Systems

### `cascade_heal_on_death`
- **Schedule**: `FixedUpdate`, in `DeathPipelineSystems::EmitHeal`.
- **run_if**: `hazard_active(HazardKind::Cascade)` + `in_state(NodeState::Playing)`.
- **Behavior**:
  1. Reads `CellDestroyed` messages.
  2. For each, issues `Quadtree::query_circle(destroyed.position, config.neighbor_radius)` via `rantzsoft_physics2d`.
  3. Filters the returned entities by `Query<Entity, (With<Cell>, Without<Dead>)>` and excludes `destroyed.entity` itself.
  4. For each remaining cell, emits `HealDealt<Cell> { target, amount: base_heal + heal_per_level * (stack - 1), cap: HealCap::Starting, source: Some("hazard:cascade".into()), .. }`.
- **Ordering**: `.after(DeathPipelineSystems::ApplyKill)` — `CellDestroyed` populated. `.before(DeathPipelineSystems::ApplyHeal)` — heal applied same tick.

## Pipeline position (dmg crate)

- **Trigger**: `CellDestroyed` (derived from `rantzsoft_dmg::Destroyed<Cell>` via the cells-domain companion emitter).
- **Emits**: `HealDealt<Cell>` in `DeathPipelineSystems::EmitHeal`.
- **Cap**: `HealCap::Starting` — nearby cells can only be healed up to their starting HP (or `Hp.max` if set, e.g., by Momentum).
- **No** `DamageDealt<T>` / `DamageBoostStack` / `VulnerableStack` involvement.

## Stacking Behavior

Linear: `heal_amount = base_heal + heal_per_level * (stack - 1)`.

| Stack | Heal per neighbor | Notes |
|-------|-------------------|-------|
| 1 | 10 | Noticeable |
| 3 | 20 | Cluster kills become costly |
| 5 | 30 | Cluster kills may heal more than chip damage inflicts |

`neighbor_radius` is stack-independent — only heal amount scales.

## Cross-Domain Dependencies
- **cells**: Reads `CellDestroyed` (cells-domain message with world-space position snapshot).
- **physics (`rantzsoft_physics2d`)**: `Quadtree::query_circle(center, radius)` — THE spatial query primitive. Works uniformly for static grid cells and animated boss cells.
- **damage crate (`rantzsoft_dmg`)**: Upstream `Destroyed<Cell>` trigger; emits `HealDealt<Cell>`.

## Expected Behaviors (for test specs)

1. **Adjacent grid cells healed at stack 1** — 3×3 grid at spacing 60, `neighbor_radius: 70`; center destroyed; 4 `HealDealt<Cell> { amount: 10.0 }` messages to the 4 orthogonal neighbors (distance 60 < 70). Diagonals (distance ≈ 85 > 70) NOT targeted.
2. **Heal scales with stack** — stack 3: 4 × `HealDealt<Cell> { amount: 20.0 }`.
3. **Corner cell has fewer in-range neighbors** — corner at (0,0) destroyed; only 2 cells within radius (right + below): 2 heal messages.
4. **Dead cells not healed** — `Dead` marker filtered out of the query results.
5. **Self excluded** — the destroyed entity is not a target of its own heal.
6. **Boss cluster off-grid** — 4 cells in a translated animated cluster (no grid slot); one destroyed; remaining 3 healed if within radius — same code path, no special boss case.
7. **Multiple deaths same tick, shared neighbor** — two cells destroyed, both within radius of a cell A: A gets 2 heal messages that tick.

## Edge Cases
- **Cascade + Tether**: Tether spreads non-lethal damage → weakened cells eventually die → Cascade heals their neighbors, potentially including the Tether partner. Intended "trap synergy".
- **Cascade + Fracture**: Fracture spawns debris at death sites. Cascade runs in `EmitHeal`, Fracture spawns `.after(ApplyKill)` in a different set — the debris isn't counted in Cascade's query because it doesn't exist yet when Cascade runs.
- **Cascade does NOT chain**: healing a cell does not trigger Cascade. Only cell DEATH triggers it.
- **Ghost cells (Echo Cells) destroyed**: their deaths also trigger Cascade (they emit `Destroyed<Cell>`, which produces `CellDestroyed`).
- **HP over-heal**: `HealCap::Starting` clamps — cells heal up to starting HP (or `Hp.max` if Momentum or Volatility has raised it).
- **Radius tuning**: `neighbor_radius` picked so standard grid spacing catches exactly the 4 orthogonal neighbors. Animated boss cells use the same radius — tune per-layout if needed via RON.
- **Cleanup**: `CascadeConfig` removed at run end; no per-entity state.

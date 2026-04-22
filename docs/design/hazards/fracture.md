# Hazard: Fracture

## Game Design

Destroyed cells spawn low-HP debris at offset positions around the death site. `2 + 1/level` debris per death, capped at 4. Debris skips slots already occupied by another cell. Clearing never truly clears — debris fills the space. Player must manage cleanup efficiently; AoE and chains shine here.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct FractureConfig {
    pub base_splits: u32,       // 2
    pub splits_per_level: u32,  // 1
    pub debris_radius: f32,     // e.g., 60.0 — offset distance from victim to each of the 4 candidate slots
    pub cell_half_size: f32,    // e.g., 20.0 — occupancy-check radius for skip-if-overlapping
}
```

Populated from `HazardTuning::Fracture`.

## Components

### `FractureDebris`

```rust
#[derive(Component, Debug, Default)]
pub(crate) struct FractureDebris;
```

Marker on spawned debris. Lets other systems identify debris (e.g., for Fracture-recursion filtering if desired).

## Messages
**Reads**: `CellDestroyed { entity, position }` (cells-domain companion to `rantzsoft_dmg::Destroyed<Cell>`).
**Sends**: None as messages. Spawns debris via `Cell::builder().debris(world_position, hp: 1.0).spawn()` through `Commands`.

## Systems

### `fracture_on_death`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyKill)`, `.after(cascade_heal_on_death)` (so Cascade evaluates original neighbors before debris appears).
- **run_if**: `hazard_active(HazardKind::Fracture)` + `in_state(NodeState::Playing)`.
- **Behavior**: For each `CellDestroyed`:
  1. Compute desired count: `(base_splits + splits_per_level * (stack - 1)).min(4)`.
  2. Generate the 4 offset positions in fixed order `[(+r, 0), (-r, 0), (0, +r), (0, -r)]` relative to `destroyed.position` (where `r = debris_radius`).
  3. Take the first `count` offsets whose target position is unoccupied — occupancy checked via `Quadtree::query_circle(offset_pos, cell_half_size)` (empty result = unoccupied).
  4. Spawn one debris cell per taken offset: `Cell::builder().debris(pos, 1.0).spawn()` (each debris has 1 HP regardless of stack; `FractureDebris` marker attached).

## Pipeline position (dmg crate)

- **Trigger**: reads `CellDestroyed` from the cells domain (companion to `rantzsoft_dmg::Destroyed<Cell>`) — every cell death produces a message; Fracture reacts.
- **Not a damage emitter or mutator.** Fracture does NOT participate in any `DeathPipelineSystems` set. It spawns debris via the cells-domain builder after the kill has already applied.
- **Ordering**: runs `.after(DeathPipelineSystems::ApplyKill)` so the primary cell is fully destroyed before debris spawn logic evaluates the scene.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` involvement.

## Stacking Behavior

Linear count scaling, capped at 4: `split_count = (base_splits + splits_per_level * (stack - 1)).min(4)`.

| Stack | Debris count | Notes |
|-------|--------------|-------|
| 1 | 2 | Two 1-HP cells per kill |
| 2 | 3 | Three per kill |
| 3 | 4 | Fills all four offset slots |
| 4+ | 4 (capped) | Stack beyond 3 has no effect on count |

Fracture caps debris spawns at 4 per death. Stack values beyond the cap (stack 5+) do not add more debris per death; they have no effect on spawn count. If the design later wants higher counts (e.g., diagonals for stack 5+), expand the offset set.

Debris HP is always 1 — stack controls quantity, not survivability. The cap prevents pathological fills; design intent is overwhelming-volume-of-trivial-cells, not tanky-debris.

## Cross-Domain Dependencies
- **cells**: `Cell::builder().debris(...)`. Reads `CellDestroyed`.
- **physics (`rantzsoft_physics2d`)**: `Quadtree::query_circle` for occupancy checks. Same spatial primitive used by Cascade/Diffusion — works uniformly for static grid cells and animated boss cells.
- **damage crate (`rantzsoft_dmg`)**: Upstream `Destroyed<Cell>` trigger.

## Expected Behaviors (for test specs)

1. **Two debris spawn at stack 1** — stack 1, cell at world `(100, 100)` destroyed, `debris_radius: 60`, all 4 offset slots empty: 2 debris spawned at `(160, 100)` and `(40, 100)` (first two offsets in fixed order).
2. **Four debris spawn at stack 3** — stack 3, all 4 offset slots empty: 4 debris spawned.
3. **Cap at stack 5** — stack 5 still spawns only 4 debris max.
4. **Skips occupied slot** — stack 3, one slot has a cell within `cell_half_size` of the offset position: that slot skipped, 3 debris spawned (one per remaining empty slot).
5. **No spawns if all slots occupied** — 0 debris.
6. **Debris always 1 HP regardless of stack** — stack 3 debris has HP 1, not 3.
7. **Works for boss-cell death** — boss cell destroyed at `(250.4, 93.1)` while moving: debris spawns at offsets from that position, not from a grid slot. Occupancy check still filters overlapping bosses.

## Edge Cases
- **Fracture + Momentum**: 1-HP debris + non-lethal hit → Momentum adds HP → 2× starting HP triggers Momentum split → chain reactions. Terrifying at high stacks.
- **Fracture + Volatility**: uncleared debris grows over time (Volatility timer) — trivial becomes 2-3 HP.
- **Recursive fracture**: `Destroyed<FractureDebris>` events also trigger Fracture by default (no filter). If playtesting shows cascades are too punishing, filter `With<FractureDebris>` from the read.
- **Position selection is deterministic (fixed order)**: no RNG. If count < 4 (e.g., 2), the first 2 offsets (right, left) are used preferentially. Future tuning could introduce seeded `GameRng` for pick-N-of-4, but fixed order is sufficient.
- **Position conflicts with Echo Cells**: Echo Cells delays 1.5s — no same-frame conflicts. Cross-frame conflicts are cells-domain concerns.
- **Out-of-bounds offsets**: if a debris offset lies outside the play area, the cells-domain spawn path rejects it (same as any other out-of-bounds spawn). No special-case in Fracture.
- **Cleanup**: `FractureConfig` removed at run end. Debris entities despawn with node via cleanup markers.

# Hazard: Diffusion

## Game Design

Incoming damage to a cell is shared with nearby cells. The original target takes reduced damage; cells within concentric range bands (rings) each receive a share. Creates a "damage sponge" effect at high stacks — clusters become nearly impenetrable.

**Stacking formula**: `share_percent = 20 + 10 * (stack - 1)` (clamped to ≤ 95%). Target takes `(100 - share) %` of the original damage; each ring takes its share of `share %` of original damage, divided across all cells in that ring.

**Cascade depth**: `depth = 1 + floor((stack - 1) / 5)`. Stack 1-5 → depth 1, stack 6-10 → depth 2, stack 11+ → depth 3+.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct DiffusionConfig {
    pub base_share_percent: f32,        // 20.0
    pub share_per_level_percent: f32,   // 10.0
    pub depth_increase_interval: u32,   // 5
    pub max_share_percent: f32,         // 95.0
    pub ring_radius: f32,               // e.g., 70.0 — ring N spans [N * r, (N+1) * r]
}
```

Populated from `HazardTuning::Diffusion`.

## Components
None.

## Messages
**Reads**: `DamageDealt<Cell>` (from `rantzsoft_dmg`).
**Sends**: `DamageDealt<Cell>` — mutates the target's damage downward and emits additional messages for cells in the rings. All happens inside the `MutateDamage` chain.

Post-TODO #1, Diffusion lives in `mutators/hazards/diffusion/` and participates in `DeathPipelineSystems::MutateDamage` as a `MessageMutator<DamageDealt<Cell>>`.

## Systems

### `diffusion_mutate_damage`
- **Schedule**: `FixedUpdate`, in `DeathPipelineSystems::MutateDamage`.
- **run_if**: `hazard_active(HazardKind::Diffusion)` + `in_state(NodeState::Playing)`.
- **Behavior**: Processes each `DamageDealt<Cell>`:
  1. Look up the victim's world position via `Position2D` (or use the target position from the damage message if it carries one).
  2. Computes `share = (base_share_percent + share_per_level_percent * (stack - 1)).min(max_share_percent)`.
  3. Computes `depth = 1 + (stack - 1) / depth_increase_interval`.
  4. Builds concentric range bands via `rantzsoft_physics2d::Quadtree::query_circle`: for ring N in 1..=depth, `cells_in_ring_N = query_circle(victim_pos, (N+1) * ring_radius) - query_circle(victim_pos, N * ring_radius)`. Target + dead cells filtered out.
  5. **Flat share per ring, not per-path**: each ring gets a single uniform share of the previous ring's total; a cell in ring N takes damage ONCE at that ring's share, regardless of how many paths reach it. Formula: `per_cell_ring_N = previous_ring_total * share_frac / current_ring_size`. Flat-share-per-ring is correct for uniform layouts and boss clusters alike; if a "diamond" layout would have reached a cell via multiple paths in a grid-BFS version, range bands already handle that naturally (one ring, one share).
  6. Reduces the target `DamageDealt<Cell>` amount to `original * (1 - share/100)`.
  7. Emits additional `DamageDealt<Cell>` for each ring cell, with source `"hazard:diffusion"`.
- **Ordering**: In `MutateDamage` set. Must run before `ApplyVulnerable` and `ApplyDamage` so the split damages flow through the rest of the chain.

## Pipeline position (dmg crate)

- **Pre-apply damage mutator** in `DeathPipelineSystems::MutateDamage`.
- Reads `DamageDealt<Cell>`, transforms the original message's amount, emits additional `DamageDealt<Cell>` for cells in the rings. Result feeds `ApplyVulnerable` → `ApplyDamage`.
- Lives in `mutators/hazards/diffusion/` (post-TODO #1 consolidated domain).
- Uses the `MessageMutator<DamageDealt<Cell>>` pattern from `rantzsoft_dmg`.
- **No** `HealDealt<T>` / `DamageBoostStack` interaction.

## Stacking Behavior

| Stack | Share | Target receives | Cascade depth |
|-------|-------|-----------------|---------------|
| 1 | 20% | 80% | 1 |
| 2 | 30% | 70% | 1 |
| 5 | 60% | 40% | 1 |
| 6 | 70% | 30% | 2 |
| 9 | 95% (clamped) | 5% | 2 |

## Cross-Domain Dependencies
- **cells**: Reads `Position2D` (target world position).
- **physics (`rantzsoft_physics2d`)**: `Quadtree::query_circle(center, radius)` — THE spatial query. Works uniformly for static grid cells and animated boss cells.
- **damage crate (`rantzsoft_dmg`)**: Mutates `DamageDealt<Cell>` in `MutateDamage`.

## Expected Behaviors (for test specs)

1. **Damage shared at stack 1** — 3×3 grid at spacing 60, `ring_radius: 70`, bolt deals 50 to center, 2 living cells within ring 1: target receives 40 (80%), each ring-1 cell receives 5 (20%/2).
2. **Share scales with stack** — stack 3 (share 40%): target with 3 ring-1 cells, 60 damage: target receives 36; each ring-1 cell 8.
3. **Depth ≥ 2 at stack 6** — stack 6: depth 2. Ring-2 band `[70, 140]` catches cells at distance ~120; they take the ring-2 share.
4. **Isolated cell receives full damage** — cell with no cells within ring 1 (boss detached from any cluster): no split, full damage to target.
5. **System skipped when Diffusion inactive** — `hazard_active(Diffusion) = false`: `DamageDealt<Cell>` passes through unmodified.
6. **Share clamped at `max_share_percent`** — stack 9 (share would be 100%): clamped to 95%; target always takes at least 5%.
7. **Boss cluster off-grid** — 4 cells in an animated cluster (no grid slots); hit one: the other 3 within ring 1 take the share evenly — same code path.
8. **Ring bands exclude inner cells** — a cell at distance 60 is in ring 1 only; it is NOT in ring 2's band (subtraction of concentric queries).

## Edge Cases
- **Diffusion + Sympathy**: Sympathy reads `DamageDealt<Cell>` AFTER Diffusion mutates (Sympathy runs in `EmitHeal`, downstream of `MutateDamage`). Sympathy heals based on post-split damage — intentional; makes Sympathy slightly less potent when Diffusion is active.
- **Diffusion + Tether**: Both mutate damage. Tether also lives in `mutators/hazards/tether/` post-TODO #1. Ordering within `MutateDamage` chain is set by `wire_damage_chain` (TODO #1) — Diffusion first, then Tether; deterministic.
- **Cascade depth overflow**: at very high stacks, rings can reach most of the level. Each ring's share attenuates naturally by count.
- **Already-destroyed cells**: the range query is filtered by `Without<Dead>`, so shared damage doesn't target corpses.
- **Self-referential**: the target entity is excluded from the ring queries.
- **Radius tuning**: `ring_radius` picked so standard grid spacing catches orthogonal neighbors in ring 1. Boss layouts may want a different ring size — tune per-layout via RON.
- **Cleanup**: `DiffusionConfig` removed at run end.

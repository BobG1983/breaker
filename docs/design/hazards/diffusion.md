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

Diffusion is implemented as a two-system pipeline split across the
`rantzsoft_dmg` `DmgSystems` chain:

### `diffusion_reduce_primary`
- **Schedule**: `FixedUpdate`, in `DmgSystems::MutateDamage`.
- **run_if**: `hazard_active(HazardKind::Diffusion)` + `in_state(NodeState::Playing)`.
- **Behavior**: For each `DamageDealt<Cell>` message in the current frame:
  1. Resolve the ring's `instance_id` — if the message `source` starts with
     `"hazard:diffusion:"`, parse the suffix as `u64` and reuse that
     instance's visited-set. Otherwise allocate a fresh id via
     `DiffusionInstances.next_id` and seed the visited-set with `msg.target`.
  2. Snapshot eligible neighbors: live cells in adjacency range
     (`ADJACENCY_RADIUS_SQ`) of `msg.target`, excluding visited, dead, and
     invulnerable cells.
  3. If no candidates → pass-through (no reduction, no pending emission).
  4. Otherwise: compute `shared = msg.amount * share_frac`,
     `msg.amount *= 1 - share_frac`, and push a `PendingEmission` onto
     `PendingDiffusionEmissions.queue` with
     `attributed_to = msg.attributed_to.or(msg.dealer)` so kill
     attribution travels to downstream ring emissions.
- **Effect on pipeline**: The reduced primary message continues through
  `ApplyVulnerable` → `ApplyDamage` this same frame; the share is queued for
  next-frame emission in `PostApply`.

### `diffusion_emit_rings`
- **Schedule**: `FixedUpdate`, in `DmgSystems::PostApply`.
- **run_if**: `hazard_active(HazardKind::Diffusion)` + `in_state(NodeState::Playing)`.
- **Behavior**: Drains `PendingDiffusionEmissions.queue`:
  1. Query `Invulnerable` on `pending.target` — if currently invulnerable,
     skip the entire emission (unified ripple-source invulnerability rule
     shared with Tether and Echo Strike).
  2. Compute `per_neighbor = pending.shared / pending.candidate_neighbors.len()`.
     If `per_neighbor < 1.0`, skip the whole ring (attenuation floor,
     inclusive at 1.0).
  3. Emit one `DamageDealt<Cell>` per neighbor with
     `source = "hazard:diffusion:{instance_id}"`, `dealer = None`, and
     `attributed_to = pending.attributed_to`. These sibling messages
     traverse the full damage pipeline on the next `FixedUpdate` tick
     (1-frame delay).
  4. Insert each emitted neighbor into `instances.visited[instance_id]`
     so subsequent BFS hops dedupe.

### State resources

- **`DiffusionInstances`** (`Resource`): `{ next_id: u64, visited: HashMap<u64,
  HashSet<Entity>> }`. Tracks per-instance visited-sets. Reset on
  `OnExit(NodeState::Playing)` via `reset_diffusion_state`.
- **`PendingDiffusionEmissions`** (`Resource`): `{ queue: Vec<PendingEmission> }`.
  Handoff queue between `diffusion_reduce_primary` (writer) and
  `diffusion_emit_rings` (drainer). Drained on each tick + on
  `OnExit(NodeState::Playing)`.

### `reset_diffusion_state`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: Resets both `DiffusionInstances` and
  `PendingDiffusionEmissions` to `Default::default()` so no state leaks
  across nodes.

## Pipeline position (dmg crate)

- **Mutator** in `DmgSystems::MutateDamage` (`diffusion_reduce_primary`) →
  primary message continues through `ApplyVulnerable` → `ApplyDamage` with
  reduced amount.
- **Ripple emitter** in `DmgSystems::PostApply` (`diffusion_emit_rings`) →
  ring siblings emitted with 1-frame delay. They traverse the full
  pipeline on Frame N+1.
- **PostApply ordering**: `diffusion_emit_rings → tether_emit_partner →
  echo_strike_emit_siblings`. Edges added by the game's
  `DmgGameOrderingPlugin`.
- **Kill attribution**: rings carry `attributed_to` from the original
  primary (`msg.attributed_to.or(msg.dealer)`), so a kill caused by ring
  damage is attributed to the original dealer via `KilledBy.killer`.
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

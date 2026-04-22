# Hazard: Sympathy

## Game Design

Damage dealt to a cell heals each nearby cell for a percentage of damage dealt. Ring 1 at base (only immediate neighbors). Every 5 stacked levels, cascade depth increases by 1 (same scaling as Diffusion). Unlike Diffusion, the target takes FULL damage — Sympathy is purely additive healing to neighbors.

**Stacking formula**: `heal_percent = 25% + 5% * (stack - 1)`. Each nearby cell in ring N is healed for `damage * heal_percent / 100.0 * attenuation(N)`.

**Cascade depth**: `depth = 1 + floor((stack - 1) / 5)`. Stack 1-5 → depth 1, stack 6-10 → depth 2.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct SympathyConfig {
    pub base_heal_percent: f32,         // 25.0
    pub heal_per_level_percent: f32,    // 5.0
    pub depth_increase_interval: u32,   // 5
    pub ring_radius: f32,               // e.g., 70.0 — ring N spans [N*r, (N+1)*r]
}
```

Populated from `HazardTuning::Sympathy`.

## Components
None. Sympathy is stateless — it reacts to damage events per-frame with no per-entity tracking.

## Messages
**Reads**: `DamageDealt<Cell>` (from `rantzsoft_dmg`) — consumed AFTER the `MutateDamage` chain has transformed the damage (so Diffusion-reduced or Tether-redirected values, if any).
**Sends**: `HealDealt<Cell>` with `HealCap::Starting` — one per ring cell per damage event. Source: `"hazard:sympathy"`. Neighbour heals must not push past pristine HP.

## Systems

### `sympathy_heal_adjacent`
- **Schedule**: `FixedUpdate`, in `DeathPipelineSystems::EmitHeal`.
- **run_if**: `hazard_active(HazardKind::Sympathy)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `DamageDealt<Cell>` AFTER the `MutateDamage` chain. For each damaged cell:
  1. Compute `heal_percent = base + per_level * (stack - 1)`.
  2. Compute `depth = 1 + (stack - 1) / depth_increase_interval`.
  3. Look up the victim's `Position2D`.
  4. For ring N in 1..=depth: `cells_in_ring_N = Quadtree::query_circle(victim_pos, (N+1) * ring_radius) - query_circle(victim_pos, N * ring_radius)`. Filter `With<Cell>, Without<Dead>` + exclude the target itself.
  5. Emit `HealDealt<Cell> { target, amount: damage * heal_percent / 100.0 * ring_attenuation(N), cap: HealCap::Starting, source: Some("hazard:sympathy".into()), .. }` for each ring cell. Ring attenuation: `ring_attenuation(N) = 1.0 / 2^(N-1)` (half per outer ring) or similar — matches Diffusion's attenuation shape so the two hazards compose predictably.
- **Ordering**: In `EmitHeal` set. Runs AFTER `MutateDamage` so Diffusion-reduced damage feeds Sympathy correctly. `.before(DeathPipelineSystems::ApplyHeal)` so the heal lands same tick.

## Pipeline position (dmg crate)

- **Post-apply reactor**: Sympathy reads `DamageDealt<Cell>` downstream of the damage-mutator chain and emits `HealDealt<Cell>`. Contrast with `MessageMutator<DamageDealt<Cell>>`-style hazards (Diffusion, Tether) that TRANSFORM the damage message inside `DeathPipelineSystems::MutateDamage` before apply.
- Both classes live in `mutators/hazards/` post-TODO #1; the distinction is pipeline position, not domain.
- **Ordering**: `sympathy_heal_adjacent` runs in `DeathPipelineSystems::EmitHeal`, ordered after the `MutateDamage` chain has run.
- **No** `DamageBoostStack` / `VulnerableStack` / `Destroyed<T>` involvement.

## Stacking Behavior

| Stack | Heal % per ring-1 cell | Cascade depth | Example: 100 dmg |
|-------|------------------------|---------------|-------------------|
| 1 | 25% | 1 | ring-1 cells heal 25 |
| 2 | 30% | 1 | 30 per ring-1 cell |
| 5 | 45% | 1 | 45 per ring-1 cell |
| 6 | 50% | 2 | ring-1: 50; ring-2: 25 |
| 11 | 75% | 3 | ring-1: 75; ring-2: 37.5; ring-3: 18.75 |

At high stacks with deep cascade, a single hit heals a wide area. Combined with Diffusion (which also gains cascade depth), clusters become nearly impenetrable. Intended "impenetrable cluster" trap.

## Cross-Domain Dependencies
- **cells**: Reads `Position2D`.
- **physics (`rantzsoft_physics2d`)**: `Quadtree::query_circle` — THE spatial query primitive. Works uniformly for static grid cells and animated boss cells.
- **damage crate (`rantzsoft_dmg`)**: Reads `DamageDealt<Cell>`; emits `HealDealt<Cell>`.

## Expected Behaviors (for test specs)

1. **Nearby cells healed at stack 1** — 3×3 grid at spacing 60, `ring_radius: 70`, stack 1; bolt deals 100 damage to center; 4 ring-1 cells each receive `HealDealt<Cell> { amount: 25.0, cap: HealCap::Starting }`. Target takes full 100 damage (no reduction).
2. **Heal percentage scales with stack 3** — stack 3 (heal 35%), 80 damage: each ring-1 cell heals 28.
3. **Cascade depth increases at stack 6** — stack 6 (depth 2): ring-1 cells heal `100 * 0.50 = 50`; ring-2 cells heal `50 * attenuation(2) = 25`.
4. **Isolated cell (no nearby cells)** — bolt deals 50 damage to a boss-tier isolated cell: no `HealDealt<Cell>` emitted.
5. **Damaged cell itself NOT healed** — target excluded from ring queries; only cells in rings 1..=depth receive heals.
6. **System skipped when inactive** — `hazard_active(Sympathy) = false`: no heal emits.
7. **Downstream of `MutateDamage`** — Diffusion reduces damage from 100 to 60 via its mutator; Sympathy heals based on 60, not 100 — ring-1 heal is `60 * 0.25 = 15` at stack 1.

## Edge Cases
- **Diffusion + Sympathy**: both gain cascade depth every 5 stacks. At depth 2+, Diffusion shares damage outward (reduces target damage) while Sympathy heals nearby based on damage dealt. Composite makes clusters extremely resilient. Ordering (Sympathy AFTER `MutateDamage`) means Sympathy heals based on Diffusion-modified damage — intentional.
- **Sympathy + Cascade**: Cascade heals when a cell is DESTROYED; Sympathy heals when a cell is DAMAGED (even if not killed). Both produce `HealDealt<Cell>`. Additive — a damaged+killed cell triggers both; neighbors get two heals.
- **Dead cells excluded**: `Without<Dead>` filter on the ring query.
- **Self-referential**: target excluded from the ring queries.
- **Zero damage**: a 0-damage `DamageDealt<Cell>` produces 0 healing (no-op).
- **Heal past max HP**: `HealCap::Starting` clamps at `Hp.starting`. Volatility's 2× cap and Sympathy's Starting cap compose cleanly.
- **Radius tuning**: `ring_radius` picked so grid spacing catches orthogonal neighbors in ring 1. Boss clusters use the same radius — tune per-layout via RON if needed.
- **Cleanup**: no per-entity state. `SympathyConfig` removed at run end.

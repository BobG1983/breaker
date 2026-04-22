# Hazard: Tether

## Game Design

Nearby cell pairs are linked with visible beams. Damage to one cell deals a percentage to its partner. Coverage determines the fraction of eligible pairs linked. Sounds helpful — isn't. Spreads non-lethal chip damage that feeds Cascade / Renewal synergies. Masters find chain-collapse sequences where Tether links cause cascading kills.

**Stacking formula**:
- Damage share: `25% + 10% * (stack - 1)`.
- Link coverage: `40% + 10% * (stack - 1)` of eligible pairs (capped at 100%).

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct TetherConfig {
    pub base_damage_percent: f32,        // 25.0
    pub damage_per_level_percent: f32,   // 10.0
    pub base_coverage_percent: f32,      // 40.0
    pub coverage_per_level_percent: f32, // 10.0
    pub pair_radius: f32,                // e.g., 70.0 — two cells within this radius are "eligible"
}
```

Populated from `HazardTuning::Tether`.

## Components

```rust
#[derive(Component, Debug)]
pub(crate) struct TetherLink {
    pub partner: Entity,
}
```

Each cell in a linked pair has a `TetherLink` pointing to the other. When one partner is destroyed, the surviving cell's `TetherLink` is removed (link breaks).

## Messages
**Reads**: `DamageDealt<Cell>` (from `rantzsoft_dmg`) — mutated inside the chain via `MessageMutator<DamageDealt<Cell>>`.
**Sends**: `DamageDealt<Cell>` — one additional entry per tethered pair hit. Source: `"hazard:tether"`. Original target's message passes through unmodified — Tether ADDS damage to the partner, does not reduce incoming.

Post-TODO #1, Tether lives in `mutators/hazards/tether/` and participates in `DeathPipelineSystems::MutateDamage` as a `MessageMutator<DamageDealt<Cell>>`.

## Systems

### `establish_tether_links`
- **Schedule**: `OnEnter(NodeState::Playing)`.
- **run_if**: `hazard_active(HazardKind::Tether)`.
- **Behavior**:
  1. Build the eligible-pair set by iterating cells and calling `Quadtree::query_circle(cell_pos, pair_radius)` — each unordered pair within `pair_radius` is eligible.
  2. Dedupe pairs (A-B == B-A).
  3. Compute `coverage_percent = base + per_level * (stack - 1)`, clamped to 100%.
  4. Using seeded `GameRng` (from run seed + node index), randomly select `coverage_percent` of eligible pairs.
  5. Insert `TetherLink { partner }` on both cells in each selected pair.

### `tether_mutate_damage`
- **Schedule**: `FixedUpdate`, in `DeathPipelineSystems::MutateDamage`. Ordering within the chain is set by `wire_damage_chain` (TODO #1) — Diffusion first, then Tether; deterministic.
- **run_if**: `hazard_active(HazardKind::Tether)` + `in_state(NodeState::Playing)`.
- **Behavior**: For each `DamageDealt<Cell>`:
  1. If the target has a `TetherLink` and the partner is alive: compute `redirect = damage * damage_percent / 100.0`.
  2. Emit an additional `DamageDealt<Cell> { target: partner, amount: redirect, source: Some("hazard:tether".into()), .. }`.
  3. Original target's damage passes through unmodified.

Note: Tether's output `DamageDealt<Cell>` is NOT itself re-tethered — only original damage events from bolt/chip trigger Tether. Loop protection is enforced by a source check (`source != "hazard:tether"`) or by running the Tether mutator once per original message.

### `cleanup_broken_tether_links`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyKill)`.
- **run_if**: `hazard_active(HazardKind::Tether)` + `in_state(NodeState::Playing)`.
- **Behavior**: For each `TetherLink`, if the partner is dead or despawned, remove the `TetherLink` from the surviving cell.

## Pipeline position (dmg crate)

- **Pre-apply damage mutator** in `DeathPipelineSystems::MutateDamage`.
- Reads `DamageDealt<Cell>`, emits additional `DamageDealt<Cell>` for partners. Original passes through. Result feeds `ApplyVulnerable` → `ApplyDamage`.
- Lives in `mutators/hazards/tether/` (post-TODO #1 consolidated domain).
- Uses the `MessageMutator<DamageDealt<Cell>>` pattern from `rantzsoft_dmg`.
- **Ordering within `MutateDamage`**: Diffusion first, then Tether (set by `wire_damage_chain`). Tether sees Diffusion-reduced damage and redirects a percentage of the reduced amount.
- **No** `HealDealt<T>` / `DamageBoostStack` interaction.

## Stacking Behavior

| Stack | Damage % to partner | Link coverage |
|-------|---------------------|---------------|
| 1 | 25% | 40% |
| 2 | 35% | 50% |
| 3 | 45% | 60% |
| 5 | 65% | 80% |
| 7 | 85% | 100% |

Coverage caps at 100% (all eligible pairs linked). Damage percent is uncapped — stack 9 = 105%, partner takes MORE damage than original hit. Intentional high-stack punishment.

## Cross-Domain Dependencies
- **cells**: Reads `Position2D` during link establishment; queries partner alive state for cleanup.
- **physics (`rantzsoft_physics2d`)**: `Quadtree::query_circle` for eligible-pair discovery. Works uniformly for static grid cells and animated boss cells.
- **damage crate (`rantzsoft_dmg`)**: Mutates `DamageDealt<Cell>` in `MutateDamage`.
- **fx**: Reads `TetherLink` for beam rendering (out of scope here).

## Expected Behaviors (for test specs)

1. **Tether links established on node start at stack 1** — 10 eligible pairs (within `pair_radius`), stack 1 (coverage 40%): 4 pairs linked (8 `TetherLink` components total).
2. **Damage redirects to partner at stack 1** — A linked to B, 100 damage to A: A receives 100 (full), B receives additional `DamageDealt<Cell> { amount: 25.0, source: "hazard:tether" }`.
3. **Damage scales with stack 3** — stack 3, 80 damage to A (linked to B): B receives 36 (45% of 80).
4. **Broken link cleaned up when partner dies** — A linked to B, B destroyed: A's `TetherLink` removed.
5. **No redirect for cells without `TetherLink`** — no additional damage emitted for unlinked cells.
6. **No recursive redirect** — Tether's own output with source `"hazard:tether"` does NOT trigger another redirect — loop protection.
7. **Works for boss-cluster pairs** — bosses animated off-grid: `query_circle` still finds pairs within `pair_radius` — same code path.

## Edge Cases
- **Tether + Cascade**: Tether spreads non-lethal damage; when a partner eventually dies, Cascade heals its neighbors. Feedback loop — Tether feeds Cascade.
- **Tether + Diffusion**: Diffusion runs first in `MutateDamage` (deterministic order via `wire_damage_chain`). Diffusion-reduced damage feeds Tether; Tether's redirected amount is based on the reduced original.
- **Bidirectional damage same frame**: A linked to B, both take damage same tick: two independent redirects fire. No infinite loop (source check).
- **Coverage randomness**: seeded `GameRng` derived from run seed + node index. Deterministic replay.
- **Mid-run stack increase**: new links established at next node's `OnEnter(NodeState::Playing)` — links are per-node. No mid-node additions (hazards don't activate mid-node anyway).
- **Radius tuning**: `pair_radius` picked so grid spacing yields orthogonal eligibility. Boss clusters use the same radius — tune via RON if needed.
- **Cleanup**: `TetherLink` components on cell entities — cleaned up at node end via cell despawn. `TetherConfig` removed at run end.

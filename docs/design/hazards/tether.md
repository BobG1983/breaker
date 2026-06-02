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
  4. Using seeded `HazardRng`, randomly select `coverage_percent` of eligible pairs.
  5. Insert `TetherLink { partner }` on both cells in each selected pair.

### `tether_emit_partner`
- **Schedule**: `FixedUpdate`, in `DmgSystems::PostApply`.
- **run_if**: `hazard_active(HazardKind::Tether)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads current-frame `DamageDealt<Cell>` messages. For each primary:
  1. Skip if the message `source` is `"hazard:tether"` (loop protection).
  2. Skip if `msg.amount <= 0.0` — the invulnerable filter has zeroed the
     primary; enforces the unified "invulnerable source → no ripple" rule
     shared with Diffusion and Echo Strike.
  3. Skip if the target has no `TetherLink`.
  4. Compute `partner_amount = msg.amount * damage_pct / 100.0`.
  5. Emit `DamageDealt<Cell> { target: partner, amount: partner_amount,
     source: Some(SourceId::from("hazard:tether")), dealer: None,
     attributed_to: msg.attributed_to.or(msg.dealer), .. }`. The partner
     sibling traverses the full damage pipeline on the next `FixedUpdate`
     tick (1-frame delay).

Loop protection is via the `source == "hazard:tether"` check; the partner
emission carries that source so a same-target feedback cycle never ignites.

### `cleanup_broken_tether_links`
- **Schedule**: `FixedUpdate`, `.after(DmgSystems::ApplyKill)`.
- **run_if**: `hazard_active(HazardKind::Tether)` + `in_state(NodeState::Playing)`.
- **Behavior**: For each `TetherLink`, if the partner is dead or despawned, remove the `TetherLink` from the surviving cell.

## Pipeline position (dmg crate)

- **Ripple emitter** in `DmgSystems::PostApply`. Reads the post-mutation,
  post-invulnerable-filter, post-apply `DamageDealt<Cell>` message and emits
  one partner sibling per tethered primary. The sibling is written into the
  same `Messages<DamageDealt<Cell>>` buffer and traverses the FULL damage
  pipeline on the next `FixedUpdate` tick (1-frame delay).
- **Loop protection** via `source == "hazard:tether"` — the partner sibling
  carries that source, so subsequent PostApply passes skip it.
- **Invulnerable skip** via `msg.amount <= 0.0` — if the primary was
  zeroed by `invulnerable_filter`, no partner emission happens.
- **Kill attribution** travels via `attributed_to = msg.attributed_to.or(msg.dealer)`
  on the partner emission, so a kill caused by partner damage attributes
  to the original dealer via `KilledBy.killer`.
- **No** `HealDealt<T>` / `DamageBoostStack` interaction.
- `TETHER_SENTINEL` constant and `TetherRedirectBuffer` resource are
  deleted — the partner emission now writes inline via
  `ResMut<Messages<DamageDealt<Cell>>>`.

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
- **Coverage randomness**: seeded `HazardRng`. Deterministic replay.
- **Mid-run stack increase**: new links established at next node's `OnEnter(NodeState::Playing)` — links are per-node. No mid-node additions (hazards don't activate mid-node anyway).
- **Radius tuning**: `pair_radius` picked so grid spacing yields orthogonal eligibility. Boss clusters use the same radius — tune via RON if needed.
- **Cleanup**: `TetherLink` components on cell entities — cleaned up at node end via cell despawn. `TetherConfig` removed at run end.

# Node Type Differentiation

How the three node types differ in content, difficulty, and pacing.

## Timer Behavior

All nodes have timers. Timer duration tightens as tier increases — there is no timer-free node type. The timer creates universal pacing pressure; node types differ in *content*, not in whether time matters.

## Passive Nodes

Clearable layouts with no hostile cells. The challenge comes from layout complexity (grid size, cell arrangement, portal nesting) and timer pressure. No cells actively fight back.

Examples: standard brick grids, portal-nested layouts, multi-hit tough cells in complex arrangements.

## Active Nodes

Contain "active cells" — cells that fight back (projectiles, shields, movement, spawning). These nodes layer offensive threats on top of the base clearing challenge.

Active cell ratio increases with tier (controlled by `active_ratio` in `TierDefinition`).

## Boss Nodes

Unique encounters with special mechanics that don't appear in normal nodes. These are hand-crafted experiences at the end of each tier.

Examples: enemies shaped like characters, falling brick patterns, locked cores requiring specific sequences.

## HP Scaling

HP scales via the `Toughness` enum (`Weak`, `Standard`, `Tough`) and the exponential formula in `ToughnessConfig`. Cells are assigned a toughness level at generation time; `ToughnessConfig.boss_multiplier` applies an extra HP multiplier on boss nodes. Portals are exempt from HP scaling.

`TierDefinition` no longer carries an `hp_mult` field — all HP scaling parameters live in `ToughnessConfig` (RON-tunable via `defaults.toughness.ron`).

This keeps early-tier nodes feeling snappy while tough cells in later tiers become meaningfully harder to break.

## Complexity Scaling

As tiers progress:
- Grid sizes increase
- More cell types are introduced (per `introduced_cell_types` in `TierDefinition`)
- Portal nesting adds depth
- Active cell ratio rises

## Content Gating

Layouts belong to a `NodePool` (Passive, Active, or Boss). The node sequence generator assigns layouts from the matching pool. This ensures passive nodes never accidentally contain active cells, and boss encounters only appear at tier boundaries.

## Dismissed Alternatives

- **Timer-free passive nodes**: Rejected. Universal timer pressure is a core pacing mechanic. Passive nodes are easier because of content, not because of missing pressure.
- **Per-node-type timer multipliers**: Not needed. Tier-level timer tightening provides sufficient difficulty scaling.
- **Run-end dead air concern**: Intentional. The pause between run end and next action provides tension release and time to review stats, seed, and flux.

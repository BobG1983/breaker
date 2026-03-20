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

HP multiplier (from `TierDefinition`) applies only to "tough" cells — cells that are inherently durable. Normal cells always stay at base HP. Portals are exempt from HP scaling.

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

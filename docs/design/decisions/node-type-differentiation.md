# Node Type Differentiation

How the three node types differ in content, difficulty, and pacing.

For the implementation details (types, HP pipeline, tier config), see `docs/architecture/node-types.md`.

## Timer Behavior

All nodes have timers. Timer duration tightens as tier increases — there is no timer-free node type. The timer creates universal pacing pressure; node types differ in *content*, not in whether time matters.

## Passive Nodes

Clearable layouts with no hostile cells. The challenge comes from layout complexity (grid size, cell arrangement, portal nesting) and timer pressure. No cells actively fight back.

## Active Nodes

Contain "active cells" — cells that fight back (projectiles, shields, movement, spawning). These nodes layer offensive threats on top of the base clearing challenge. Active cell ratio increases with tier.

## Boss Nodes

Unique encounters with special mechanics that don't appear in normal nodes. Hand-crafted experiences at the end of each tier.

## Content Gating

Layouts belong to a pool (Passive, Active, or Boss). The node sequence generator assigns layouts from the matching pool. This ensures passive nodes never accidentally contain active cells, and boss encounters only appear at tier boundaries.

## Dismissed Alternatives

- **Timer-free passive nodes**: Rejected. Universal timer pressure is a core pacing mechanic. Passive nodes are easier because of content, not because of missing pressure.
- **Per-node-type timer multipliers**: Not needed. Tier-level timer tightening provides sufficient difficulty scaling.
- **Run-end dead air concern**: Intentional. The pause between run end and next action provides tension release and time to review stats, seed, and flux.

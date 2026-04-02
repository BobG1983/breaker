## Stable
- [Bolt Velocity2D system map](system-map-bolt-velocity2d.md) — all systems writing Velocity2D on bolts, sets, and ordering; updated 2026-03-31 for builder migration: prepare_bolt_velocity and PrepareVelocity set eliminated; gravity_well/attraction ordering anchors need re-verification
- [Speed boost checker ordering](speed-boost-checker-ordering.md) — speed_boost.rs now has inline recalculate_velocity (Option B resolved); ordering chain and historical lag analysis
- [Cross-domain ordering map](cross-domain-ordering-map.md) — all system sets defined in one domain but referenced in another for ordering; bolt↔breaker↔effect circular deps; analyzed 2026-04-01 for game-crate-splitting feasibility

## Session History
See [ephemeral/](ephemeral/) — not committed.

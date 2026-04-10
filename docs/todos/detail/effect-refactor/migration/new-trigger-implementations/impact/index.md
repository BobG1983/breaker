# Impact Trigger Implementations

## Types
- [types.md](types.md) — existing collision messages and ImpactTarget enum

## Local Trigger (on participants)
- [on_impacted.md](on_impacted.md) — 6 sub-systems, one per collision pair, walks each participant with `Impacted(other_kind)`

## Global Trigger (on all entities with BoundEffects/StagedEffects)
- [on_impact_occurred.md](on_impact_occurred.md) — 6 sub-systems, one per collision pair, walks all entities with `Impact(kind)` for both participant kinds

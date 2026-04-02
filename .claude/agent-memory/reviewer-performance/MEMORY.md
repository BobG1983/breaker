# Memory

- [Entity scale expectations](entity_scale.md) — Breaker: 1 entity, Bolt: 1–few, Cells: ~50–200 grid, Walls: few fixed
- [ActiveSizeBoosts archetype fragmentation](archetype_activesizeboosts.md) — Optional on Breaker; fragmentation is academic at 1 entity
- [SpatialData optional fields](spatialdata_optional_fields.md) — scale + previous_scale added; bolt count is tiny so negligible
- [DispatchInitialEffects QueryState pattern](dispatch_initial_effects_querystate.md) — world.query_filtered() inside Command::apply() is acceptable at chip-equip frequency
- [sync_breaker_scale schedule](sync_breaker_scale_schedule.md) — Runs in Update (visual sync); FixedUpdate in tests only — mismatch is intentional, acceptable
- [BreakerBuilder spawn pattern](breaker_builder_spawn_pattern.md) — ~30 components in one spawn(), Vec<RootEffect> clone, Primary/Extra archetype split — all spawn-time, all acceptable at 1 entity

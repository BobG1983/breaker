# Memory

- [Entity scale expectations](entity_scale.md) — Breaker: 1 entity, Bolt: 1–few, Cells: ~50–200 grid, Walls: few fixed
- [ActiveSizeBoosts archetype fragmentation](archetype_activesizeboosts.md) — Optional on Breaker; fragmentation is academic at 1 entity
- [SpatialData optional fields](spatialdata_optional_fields.md) — scale + previous_scale added; bolt count is tiny so negligible
- [DispatchInitialEffects QueryState pattern](dispatch_initial_effects_querystate.md) — world.query_filtered() inside Command::apply() is acceptable at chip-equip frequency
- [sync_breaker_scale schedule](sync_breaker_scale_schedule.md) — Runs in Update (visual sync); FixedUpdate in tests only — mismatch is intentional, acceptable
- [BreakerBuilder spawn pattern](breaker_builder_spawn_pattern.md) — ~30 components in one spawn(), Vec<RootEffect> clone, Primary/Extra archetype split — all spawn-time, all acceptable at 1 entity
- [BoltBuilder archetype fragmentation](bolt_builder_archetype_fragmentation.md) — 6-10 conditional inserts in spawn_inner produce multiple archetypes per bolt variant — spawn-time, acceptable at 1-few bolts
- [sync_bolt_scale schedule](sync_bolt_scale_schedule.md) — Runs in FixedUpdate unconditionally; cheap at 1 bolt; no change-detection guard needed at current scale
- [spawn_bolt remove/insert Assets](spawn_bolt_remove_insert_assets.md) — remove_resource+insert_resource for Assets<Mesh/ColorMaterial> is correct pattern for World-exclusive system; spawn-time only
- [propagate_breaker_changes hot reload](propagate_breaker_changes_hot_reload.md) — 3 chained .insert() splits for ~30 components; is_changed() guard means zero per-frame cost in production
- [WallRegistry data pattern](wall_registry_pattern.md) — HashMap<String, WallDefinition> Resource; seed() clones are startup-only; 4 walls, no per-frame cost
- [WallBuilder spawn pattern](wall_builder_spawn_pattern.md) — typestate builder; ~3-4 spawns at node start; all allocations spawn-time; legacy spawn_walls system coexists acceptably
- [update_pause_menu_colors pattern](pause_menu_colors_pattern.md) — 2-entity Update query gated on is_time_paused; unconditional write is fine at this scale; no allocations
- [cleanup_on_exit double registration](cleanup_on_exit_double_registration.md) — OnEnter(NodeState::Teardown) + OnEnter(RunState::Teardown) both register cleanup_on_exit<NodeState>; intentional safety net, zero per-frame cost
- [handle_pause_input params](handle_pause_input_params.md) — ResMut<NodeOutcome> + MessageWriter in Update; gated by run_if; no cross-schedule parallelism conflict with FixedUpdate lifecycle writers

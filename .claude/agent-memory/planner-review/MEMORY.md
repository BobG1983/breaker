# Planner-Review Memory

## Common Spec Issues
- [pattern_message_field_additions.md](pattern_message_field_additions.md) — Message struct field additions miss the sender's test construction sites
- [pattern_deferred_commands_observer.md](pattern_deferred_commands_observer.md) — Bevy observer commands.insert() is deferred; can't read back in same call
- [pattern_ron_hp_scaling_scenario_impact.md](pattern_ron_hp_scaling_scenario_impact.md) — RON HP scaling changes scenario dynamics; frame limits may need updating
- [pattern_cross_system_component_preservation.md](pattern_cross_system_component_preservation.md) — "Don't touch X" specs must enumerate ALL components in the category
- [pattern_plugin_registration_in_impl_spec.md](pattern_plugin_registration_in_impl_spec.md) — New systems need plugin.rs registration (schedule, ordering, run_if, set)
- [pattern_copy_derive_removal_cascade.md](pattern_copy_derive_removal_cascade.md) — Removing Copy from an enum breaks all pattern-match sites that relied on implicit copy
- [pattern_default_vs_ron_test_values.md](pattern_default_vs_ron_test_values.md) — Specs citing RON values but tests using Default trait values; CellConfig and PlayfieldConfig defaults differ from RON
- [pattern_onenter_resource_timing.md](pattern_onenter_resource_timing.md) — Systems reading resources inserted via deferred commands in OnEnter need explicit ordering after the inserting chain
- [pattern_onenter_deferred_resource_chain.md](pattern_onenter_deferred_resource_chain.md) — Chained OnEnter systems using commands.insert_resource() need apply_deferred between producer and consumer
- [pattern_enum_signature_migration.md](pattern_enum_signature_migration.md) — Changing enum variant signatures breaks ALL existing construction/match sites; spec must enumerate them as prerequisites
- [pattern_refactor_atomicity.md](pattern_refactor_atomicity.md) — Multi-unit refactors that move/rename types cannot compile independently; must be atomic or use re-export bridge
- [pattern_observer_mutation_vs_message.md](pattern_observer_mutation_vs_message.md) — Observers that mutate components need different query patterns than observers that write messages; spec must specify exact query
- [pattern_message_field_removal_cascade.md](pattern_message_field_removal_cascade.md) — Removing a field from a Message struct breaks ALL construction sites; specs must enumerate every file
- [pattern_onenter_deferred_resource_chain.md](pattern_onenter_deferred_resource_chain.md) — Chained OnEnter systems using commands.insert_resource() need apply_deferred between producer and consumer
- [pattern_onexit_stale_resource.md](pattern_onexit_stale_resource.md) — OnExit(StateA) systems read stale resource values if the resource is set during the NEXT state
- [pattern_pub_struct_refactor_cascade.md](pattern_pub_struct_refactor_cascade.md) — Changing a pub struct's inner type cascades across workspace crates; writer agents can't fix cross-crate breakage
- [pattern_component_type_migration_cascade.md](pattern_component_type_migration_cascade.md) — Migrating entity position type cascades through query aliases, test construction sites, scenario runner, debug/effect systems
- [pattern_scale2d_zero_panic.md](pattern_scale2d_zero_panic.md) — Scale2D::new panics on zero; expanding-radius effects must guard initial frame
- [pattern_global_component_change_detection.md](pattern_global_component_change_detection.md) — Global* components written every frame make Changed<Global*> always true, defeating incremental quadtree updates
- [pattern_cap_removal_dedup_asymmetry.md](pattern_cap_removal_dedup_asymmetry.md) — Removing highlight cap exposes asymmetric dedup: per-event systems have per-kind checks, track_node_cleared_stats does not
- [pattern_cross_domain_pure_function_visibility.md](pattern_cross_domain_pure_function_visibility.md) — Pure functions called cross-domain need explicit pub visibility and mod.rs export chain
- [pattern_dual_semantic_enum_variant.md](pattern_dual_semantic_enum_variant.md) — Enum variant reused across dispatch contexts (triggered vs passive) has ambiguous field semantics
- [pattern_test_app_resource_mismatch.md](pattern_test_app_resource_mismatch.md) — System gaining new Res/ResMut parameter requires test_app() to also init that resource
- [pattern_struct_field_addition_crosscrate.md](pattern_struct_field_addition_crosscrate.md) — Adding a field to pub struct breaks ALL struct literal sites across workspace; serde(default) only helps RON

## Domain Quirks
- `BoltHitCell` is `pub(crate)` (not `pub`) in `bolt/messages.rs` (moved from `physics/messages.rs` in 2026-03-24 spatial/physics extraction)
- `CellHealth::new()` and `is_destroyed()` are `const fn` — new methods on the same impl should follow suit where possible
- chips domain does NOT currently consume `BoltHitCell` despite messages.md listing it as a consumer

## Session History
See [ephemeral/](ephemeral/) — not committed.

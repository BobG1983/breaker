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
- [pattern_enum_signature_migration.md](pattern_enum_signature_migration.md) — Changing enum variant signatures breaks ALL existing construction/match sites; spec must enumerate them as prerequisites
- [pattern_refactor_atomicity.md](pattern_refactor_atomicity.md) — Multi-unit refactors that move/rename types cannot compile independently; must be atomic or use re-export bridge

## Domain Quirks
- `BoltHitCell` is `pub(crate)` (not `pub`) in `physics/messages.rs`
- `CellHealth::new()` and `is_destroyed()` are `const fn` — new methods on the same impl should follow suit where possible
- chips domain does NOT currently consume `BoltHitCell` despite messages.md listing it as a consumer

## Session History
See [ephemeral/](ephemeral/) — not committed.

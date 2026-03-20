# Planner-Review Memory

## Common Spec Issues
- [pattern_message_field_additions.md](pattern_message_field_additions.md) — Message struct field additions miss the sender's test construction sites
- [pattern_deferred_commands_observer.md](pattern_deferred_commands_observer.md) — Bevy observer commands.insert() is deferred; can't read back in same call
- [pattern_ron_hp_scaling_scenario_impact.md](pattern_ron_hp_scaling_scenario_impact.md) — RON HP scaling changes scenario dynamics; frame limits may need updating
- [pattern_cross_system_component_preservation.md](pattern_cross_system_component_preservation.md) — "Don't touch X" specs must enumerate ALL components in the category
- [pattern_plugin_registration_in_impl_spec.md](pattern_plugin_registration_in_impl_spec.md) — New systems need plugin.rs registration (schedule, ordering, run_if, set)

## Domain Quirks
- `BoltHitCell` is `pub(crate)` (not `pub`) in `physics/messages.rs`
- `CellHealth::take_hit()` is `const fn` — new methods on the same impl should follow suit
- chips domain does NOT currently consume `BoltHitCell` despite messages.md listing it as a consumer

## Session History
See [ephemeral/](ephemeral/) — not committed.

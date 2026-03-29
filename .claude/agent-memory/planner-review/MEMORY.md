# Planner Review Memory

## Spec Patterns (What Goes Wrong)
- [Asymmetric cap removal / dedup](pattern_cap_removal_dedup_asymmetry.md) — caps and dedup behave differently than expected
- [Component type migration cascade](pattern_component_type_migration_cascade.md) — renaming a component cascades across all callers
- [Copy derive removal cascade](pattern_copy_derive_removal_cascade.md) — removing Copy breaks implicit callers
- [Cross-domain pure function visibility](pattern_cross_domain_pure_function_visibility.md) — pub(crate) functions not visible across domain boundaries
- [Cross-system component preservation](pattern_cross_system_component_preservation.md) — systems clobbering components set by other systems
- [Dead code directory vs live](pattern_dead_code_directory_vs_live.md) — referencing files in dead-code directories as if they are live
- [Default vs RON test values](pattern_default_vs_ron_test_values.md) — tests using Default but RON overrides change behavior
- [Deferred commands / observer](pattern_deferred_commands_observer.md) — commands deferred past observer scope
- [Dual semantic enum variant](pattern_dual_semantic_enum_variant.md) — one variant carrying two semantic roles
- [Enum signature migration](pattern_enum_signature_migration.md) — changing an enum field name breaks all match arms
- [Existing tests in spec scope](pattern_existing_tests_in_spec_scope.md) — spec forgets to account for tests that already exist
- [Field type change / semantic shift](pattern_field_type_change_semantic_shift.md) — changing a field's type implies behavior change not stated in spec
- [File move / test import cascade](pattern_file_move_test_import_cascade.md) — moving a file breaks test use paths
- [Global component change detection](pattern_global_component_change_detection.md) — change detection on global components fires every frame
- [Message field additions](pattern_message_field_additions.md) — adding fields to messages breaks all construction sites
- [Message field removal cascade](pattern_message_field_removal_cascade.md) — removing message fields breaks readers
- [NodeTimer field names](pattern_nodetimer_field_names.md) — NodeTimer fields named differently than expected
- [Observer mutation vs message](pattern_observer_mutation_vs_message.md) — mutating component in observer vs sending a message
- [OnEnter deferred resource chain](pattern_onenter_deferred_resource_chain.md) — OnEnter systems creating resources that later systems depend on
- [OnEnter resource timing](pattern_onenter_resource_timing.md) — resource not yet available when OnEnter system runs
- [OnExit stale resource](pattern_onexit_stale_resource.md) — resource still holds state from previous run on re-entry
- [Plugin registration in impl spec](pattern_plugin_registration_in_impl_spec.md) — impl spec forgets to note plugin registration step
- [Pub struct refactor cascade](pattern_pub_struct_refactor_cascade.md) — making a struct's fields pub cascades to callers
- [Refactor atomicity](pattern_refactor_atomicity.md) — spec splits a rename across multiple PRs, causing a broken intermediate state
- [RON HP scaling / scenario impact](pattern_ron_hp_scaling_scenario_impact.md) — RON value changes break scenario invariants
- [Scale2D zero panic](pattern_scale2d_zero_panic.md) — Scale2D(0.0) panics in spatial2d
- [Struct field addition cross-crate](pattern_struct_field_addition_crosscrate.md) — adding a field to a public struct breaks constructors in other crates
- [Test app resource mismatch](pattern_test_app_resource_mismatch.md) — test app missing resources that production app registers
- [Rename with incomplete doc sweep](pattern_rename_incomplete_doc_sweep.md) — field rename spec lists some docs but misses others; use grep to verify all occurrences before approving

- [Missing prerequisite type gap](pattern_missing_prerequisite_type_gap.md) — spec assumes Effective* components exist from prior wave but they were never defined

## Session History
See [ephemeral/](ephemeral/) — not committed.

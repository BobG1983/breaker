## Rules
- [rule_never_run_cargo.md](rule_never_run_cargo.md) — NEVER run cargo commands — only runner agents run cargo

## Stable
- [pattern_config_default_vs_ron.md](pattern_config_default_vs_ron.md) — Rust Default may differ from RON; tests use Rust Default
- [pattern_asset_event_testing.md](pattern_asset_event_testing.md) — Testing AssetEvent<T>/Modified in Bevy 0.18
- [pattern_message_writer_in_tests.md](pattern_message_writer_in_tests.md) — Enqueuing messages in Bevy 0.18 tests
- [pattern_bundle_tuple_limit.md](pattern_bundle_tuple_limit.md) — Bevy Bundle tuple size limit workaround
- [pattern_ron_enum_variant_format.md](pattern_ron_enum_variant_format.md) — RON enum variant formatting
- [pattern_pure_rust_stub_for_tests.md](pattern_pure_rust_stub_for_tests.md) — Compilable todo!() stubs for non-Bevy modules
- [pattern_bevy_system_stub_for_tests.md](pattern_bevy_system_stub_for_tests.md) — Bevy system stubs and type alias for complex Query
- [pattern_message_capture.md](pattern_message_capture.md) — Capture MessageReader<T> into Resource for assertion
- [pattern_doc_markdown_clippy.md](pattern_doc_markdown_clippy.md) — CamelCase in doc comments needs backticks; derive Eq alongside PartialEq
- [pattern_entity_generational_ids_testing.md](pattern_entity_generational_ids_testing.md) — Bevy generational IDs: stale HashMap entries cause memory leaks, not false violations
- [pattern_observer_message_timing.md](pattern_observer_message_timing.md) — Flush commands before tick when observers write Messages for MessageReader capture

## Session History
See [ephemeral/](ephemeral/) — not committed.

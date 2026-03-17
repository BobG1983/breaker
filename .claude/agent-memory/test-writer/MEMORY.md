# Test Writer Agent Memory

## Patterns
- [pattern_asset_event_testing.md](pattern_asset_event_testing.md) — How to test AssetEvent<T>/Modified in Bevy 0.18 integration tests
- [pattern_message_writer_in_tests.md](pattern_message_writer_in_tests.md) — How to enqueue messages in Bevy 0.18 integration tests
- [pattern_bundle_tuple_limit.md](pattern_bundle_tuple_limit.md) — Bevy Bundle tuple size limit; split large spawns into spawn + entity_mut().insert()
- [pattern_ron_enum_variant_format.md](pattern_ron_enum_variant_format.md) — Double vs single paren RON format for enum variants; newtype wrapper structs for double-paren
- [pattern_pure_rust_stub_for_tests.md](pattern_pure_rust_stub_for_tests.md) — Compilable todo!() stubs for pure-Rust (non-Bevy) modules so tests fail correctly
- [pattern_bevy_system_stub_for_tests.md](pattern_bevy_system_stub_for_tests.md) — Bevy system todo!() stubs, type alias for complex Query params, adding bevy dep to non-bevy crates
- [pattern_doc_markdown_clippy.md](pattern_doc_markdown_clippy.md) — All CamelCase names and field names in doc comments need backticks (clippy::pedantic denies doc_markdown); also derive Eq alongside PartialEq where applicable

## Session History
See [ephemeral/](ephemeral/) — not committed.

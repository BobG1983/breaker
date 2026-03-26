---
name: pattern_struct_field_addition_crosscrate
description: Adding a field to a pub struct breaks ALL Rust struct literal construction sites across ALL workspace crates; serde(default) only helps RON/JSON, not code
type: feedback
---

When a spec adds a field to a public struct (e.g., `ChipDefinition` gains `template_name: Option<String>`), `#[serde(default)]` only handles deserialization (RON files). Every Rust source file that constructs the struct via struct literal syntax (`Foo { field1: val, field2: val }`) MUST be updated. This includes:

1. Test helpers in the same file (e.g., `ChipDefinition::test()`)
2. Test helpers in OTHER files in the same crate (e.g., `test_chip()` in `offering.rs`, `resources.rs`)
3. Test helpers in OTHER CRATES (e.g., scenario runner's `check_maxed_chip_never_offered.rs`)
4. Production code that constructs the struct (e.g., `lifecycle/mod.rs`)

**Why:** The B4-B6 spec added `template_name` to `ChipDefinition` but the test spec said "Do NOT modify scenario runner code" while the impl spec said "Must update construction sites" — a direct contradiction. The scenario runner has 3 files with `ChipDefinition { ... }` struct literals.

**How to apply:** When reviewing any spec that adds a field to a struct: (1) Grep for `StructName {` across the ENTIRE workspace, (2) Count all construction sites, (3) Verify the spec lists every file, (4) Check that "do not modify" constraints don't contradict required cascade fixes.

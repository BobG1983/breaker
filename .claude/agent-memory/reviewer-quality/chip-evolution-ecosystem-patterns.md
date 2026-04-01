---
name: Chip Evolution Ecosystem — Established Patterns
description: Intentional patterns from the chip-evolution-ecosystem branch review: PoolEntry pub fields, draw_offerings test-only helper, format!("{:?}") rarity display, rarity_color_* dead fields, FUTURE commented-out methods, b_entry.unwrap() in tests
type: project
---

## `PoolEntry` pub fields are intentional

`PoolEntry { pub name, pub weight, pub template_name }` in `offering/system.rs` has all fields `pub`
despite the struct being `pub(crate)`. The fields are accessed directly by the test helpers in
`offering/tests/helpers.rs` via `draw_offerings`. This is the intended design — the struct
is an internal data carrier, not an API boundary. Do NOT flag the pub fields.

## `draw_offerings` is test-only, not production duplication

`offering/tests/helpers.rs::draw_offerings` re-implements the draw loop for test isolation.
This is intentional — it lets tests drive the weighted draw primitive directly without the full
`generate_offerings` path. Do NOT flag as production duplication.

## `format!("{:?}", def.rarity)` in `spawn_chip_select.rs` is a placeholder

Line 132 of `spawn_chip_select.rs` uses debug formatting to display rarity text. This is a
known placeholder UI approach for early screen development. The `rarity_color_*_rgb` fields
in `ChipSelectConfig` exist but are not yet wired to the spawned card UI. These are open
[Debt] items, not Nits.

## `rarity_color_*_rgb` fields in `ChipSelectConfig` unused in spawn

`ChipSelectConfig` declares `rarity_color_common_rgb`, `rarity_color_uncommon_rgb`,
`rarity_color_rare_rgb`, `rarity_color_legendary_rgb` — all default-initialised but not yet
consumed by `spawn_chip_select`. This is intentional forward declaration for upcoming card
color rendering. Flag as [Debt], not [Fix].

## FUTURE commented-out `is_empty()` methods in registries are intentional

`ChipTemplateRegistry` and `EvolutionTemplateRegistry` each have a commented-out `is_empty()`
method marked `// FUTURE: may be used for upcoming phases`. These are intentional stubs, not
dead code. Do NOT flag.

## `b_entry.unwrap()` pattern in pool_and_exclusion tests is acceptable

`pool_and_exclusion.rs:78,87` calls `.unwrap()` after a `pool.iter().find()` assertion.
The unwrap is immediately preceded by `assert!(b_entry.is_some(), ...)`. This is idiomatic
test code — the assert gives the failure message, the unwrap is guaranteed safe by it.
Do NOT flag as production unwrap risk.

## `ChipCatalog::insert` duplicate-name test is documenting known behavior, not a bug

`chip_catalog.rs` includes a test `chip_catalog_insert_duplicate_name_overwrites_map_but_pushes_order`
that explicitly documents that the `order` Vec grows on duplicate inserts while the map entry
is overwritten. This is intentional documentation of the current behavior (which could produce
doubled entries in `ordered_values()`). The test is an accurate description, not a bug catch.
Do NOT flag the test as asserting intermediate state.

---
name: known_unsafe_blocks
description: Inventory of unsafe blocks in the workspace and their justification status
type: project
---

## Unsafe block inventory (as of 2026-03-28)

**Result: None found.**

The workspace lint configuration in `Cargo.toml` sets:
```
unsafe_code = "deny"
unsafe_op_in_unsafe_fn = "deny"
undocumented_unsafe_blocks = "deny"
```

No unsafe blocks exist anywhere in `breaker-game/src/`. Verified by grep across all
changed files in the Phase 1 collision cleanup diff and full-source scan.

No FFI boundaries, no raw pointer manipulation, no proc macros with untrusted input.
No `build.rs` files in any crate.

Still confirmed after Phase 3 effect system + trigger bridge changes (2026-03-28).
Still confirmed after Phase 4+5 runtime effects changes (2026-03-28, feature/runtime-effects):
attraction, chain_bolt, explode, pulse, second_wind, shockwave, spawn_phantom — no unsafe.

Still confirmed after Phase 6 changes (2026-03-29, feature/source-chip-shield-absorption):
source_chip threading, EffectSourceChip component, shield charge absorption, chain lightning
arc-based rework — no unsafe. All mem::replace usage is safe Rust (not unsafe).

Still confirmed after refactor (2026-03-30, develop post-merge, c9964b7):
23 files split into directory modules — code-only structural moves. No unsafe introduced.
Grep for "unsafe" across all .rs files: no matches.

Still confirmed after feature/missing-unit-tests (2026-03-30):
Branch adds test code only (plus one pub(super) visibility widening). No unsafe blocks added.
Workspace lint unsafe_code = "deny" remains in force.

Still confirmed after feature/scenario-coverage (2026-03-30):
New scenario runner checkers (check_aabb_matches_entity_dimensions.rs,
check_gravity_well_count_reasonable.rs, check_size_boost_in_range.rs), new frame mutation
helpers (apply_inject_mismatched_bolt_aabb, apply_spawn_extra_gravity_wells,
apply_inject_wrong_size_multiplier), and gravity_well visibility widening
(pub instead of pub(crate)) — no unsafe blocks anywhere. Grep confirmed zero matches.

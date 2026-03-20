---
name: Lint Patterns
description: Recurring cargo fmt and clippy patterns specific to this codebase
type: reference
---

## Cargo Formatting Rules
- Line wrap conditional expressions in assignments (bolt_breaker_collision.rs lines 43-44)
- Avoid multi-line Color::srgb calls - use single line after assignment operator
- Multi-line `assert!` with format args must be wrapped: condition, message, args on separate lines
- Multi-line method chains: wrap at logical points when necessary
- Long function call arguments: one argument per line when wrapping
- Import order: nested imports should be ordered (ClearRemainingCount, NodeSystems before systems)

## Clippy Patterns
- Type aliases required for complex Query filters (CellQueryFilter, BreakerQueryFilter)
- Use `.mul_add()` for floating point operations to satisfy suboptimal_flops lint
- Collapse nested `if let` with inner `if` condition into single `if let ... && ...` (collapsible_if lint)
- Keep test helper structs and functions at module level, not inside test functions (items_after_statements)
- Doc comments: wrap Rust identifiers and types in backticks (e.g., `HashMap`, `BreakerState`) for doc_markdown lint
- `private_interfaces` warning: `pub(crate)` structs in `chips/components.rs` (BoltSpeedBoost, BreakerSpeedBoost, WidthBoost, BumpForceBoost) referenced in `pub` query type aliases (breaker/queries.rs) and `pub` system functions. Fix: either make the chip components `pub`, or make the query aliases `pub(crate)`. Not an error — just a warning — but appears in bulk (9 instances) whenever chip components are `pub(crate)`.
- `E0364` re-export visibility error: Stream 4 pub→pub(crate) visibility fixes introduced a mismatch between function visibility and mod.rs `pub use` re-exports. Pattern: function is `pub(crate)` in its own file but re-exported with `pub use` in `mod.rs`. The `pub use` in mod.rs must be downgraded to `pub(crate) use` to match. Affected in phase4b2: `bolt/systems/prepare_bolt_velocity.rs:18`, `breaker/systems/bump.rs:67+133` (grade_bump, update_bump), `breaker/systems/move_breaker.rs:19`.
- `similar_names` error (clippy::pedantic): test-local bindings `cell_a_y` and `cell_b_y` are too similar. Occurs in test code at `physics/systems/bolt_cell_collision.rs:1213-1214`. Fix: rename one binding (e.g., `cell_below_y` instead of `cell_b_y`).
- `type_complexity` error: Query with 6-tuple data + filter tuple triggers this in `bolt/systems/reset_bolt.rs` (the bolt_query with Entity, Transform, BoltVelocity, Option<PiercingRemaining>, Option<Piercing>, Option<PhysicsTranslation>) and `bolt/systems/spawn_bolt.rs` (bundled tuple `(Query<&Transform, With<Breaker>>, Query<Entity, With<Bolt>>)`). Fix: extract named type aliases to `bolt/queries.rs` following the `LostQuery` pattern already there.
- `if_not_else` error (clippy::pedantic): `if !condition { A } else { B }` pattern. Fix: invert the condition to `if condition { B } else { A }`. Occurs in `chips/effects/piercing.rs:35` (`if !had_piercing`).
- `uninlined_format_args` error (clippy::pedantic): `assert!(..., "msg {}", var)` where `var` can be inlined as `{var}`. Fix: replace `{}` + trailing arg with `{var}`. Occurs in `chips/effects/mod.rs:179`.
- `dead_code` warning: `BoltHitBreaker.bolt` field in `physics/messages.rs:11` is `pub` inside a `pub(crate)` struct but never read. Recurring warning — field appears to be forward-declared for future audio/chips consumers. Not an error.
- `explicit_counter_loop` error: when an explicit `u32` counter variable is incremented in a `for` loop, clippy requires using `(0_u32..).zip(iter)` instead. Occurs in `run/systems/generate_node_sequence.rs`.
- `cast_possible_truncation` / `cast_precision_loss` errors: `usize as u32` and `u32 as f32` casts in `run/systems/generate_node_sequence.rs`. Fix: `u32::try_from(t)` for usize→u32; for u32→f32 precision loss, use `f32::from(boss_count)` (lossless for smaller u32 values, or use explicit cast with a comment if intentional).
- `dead_code` warning (chips/inventory.rs): `ChipInventory` methods `add_chip`, `stacks`, `is_maxed`, `mark_seen`, `has_seen`, `held_chips`, `maxed_chips`, `total_held` and `ChipEntry` fields `stacks`, `max_stacks`, `rarity` are unused — forward-declared for future chip consumption systems. Not errors, but appear in bulk when chip systems are not yet wired up.
- `dead_code` warning (cells/components.rs:93): `CellGridPosition` struct is never constructed — added in phase4-wave2-session4, forward-declared for future grid-position-aware systems. Not an error.
- `expect_used` warning (clippy::restriction): `u16::try_from(...).expect(...)` in `run/systems/generate_node_sequence.rs:52,58`. These are intentional — the counts are always small enough to fit u16 and the expect message explains the invariant. Not an error, but the workspace lint config enables `-W clippy::expect-used` so they appear as warnings every run.
- `collapsible_if` error (cells/definition.rs:63): `if let Some(rate) = self.behavior.regen_rate { if rate <= 0.0 || !rate.is_finite() { ... } }` — collapse to `if let Some(rate) = self.behavior.regen_rate && (rate <= 0.0 || !rate.is_finite()) { ... }`.
- `option_as_ref_deref` / `&Option<T>` error: `&Option<Res<T>>` parameters in `run/node/systems/spawn_cells_from_layout.rs:108-110` — change function signature to `Option<&Res<T>>` (or pass the inner value directly).
- `too_many_arguments` error: `spawn_cells_from_layout` at `run/node/systems/spawn_cells_from_layout.rs:127` has 9 args (limit 7). Fix: bundle related args into a helper struct or SystemParam, or split the function.
- `map_unwrap_or` error: `.map(|x| ...).unwrap_or_else(|| ...)` pattern — replace with `.map_or_else(|| fallback, |x| ...)`. Occurs at `run/systems/handle_node_cleared.rs:26`.
- `dead_code` warning (behaviors/registry.rs): `ArchetypeRegistry` methods `iter`, `values`, `len`, `is_empty` (and `clear` in test build) are never used — forward-declared for future archetype lookup systems. Not errors.
- `dead_code` warning (cells/resources.rs): `CellTypeRegistry` methods `values`, `iter`, `aliases`, `len`, `is_empty` (and `clear` in test build) are never used — forward-declared. Not errors.
- `unreachable_pub` warnings (cells/definition.rs, run/systems/generate_node_sequence.rs): `pub` items in internal modules should be `pub(crate)` or `pub(super)`. Recurring across new sessions when new public types are added.
- Format change (session 2026-03-19): single-line `assert!` now preferred when condition + message fit on one line (`cells/definition.rs:119`). Multi-argument imports collapsed to `use crate::run::{ ... }` block form (`generate_node_sequence.rs:4`). Test helper function signature collapsed to one line when short enough (`handle_node_cleared.rs:188`).
- `private_interfaces` warning (run/node/systems/spawn_cells_from_layout.rs:135): `CellSpawnContext<'_>` is `pub(crate)` but `spawn_cells_from_layout` is `pub`. Fix: downgrade function to `pub(crate)` or elevate struct to `pub`. Not an error — a warning. Added in phase4-wave2-session4. RESOLVED as of 2026-03-19l — no longer appears.
- `unreachable_pub` warnings (cells/messages.rs:9, cells/resources.rs:14): `CellDestroyed` and `CellDefaults` are `pub` structs in crate-internal modules. Clippy suggests `pub(crate)`. These are currently warnings (not errors). Added 2026-03-19l.
- `unused_imports` warning (run/node/systems/mod.rs:17): `spawn_cells_from_grid` re-exported as `pub(crate) use` but the only consumer (`debug/hot_reload/systems/propagate_node_layout_changes.rs`) imports it directly via `systems::spawn_cells_from_grid`, bypassing the re-export. Only appears in `dsclippy` (scenario runner build), not `dclippy`. Added 2026-03-19l.
- `dead_code` warnings (debug/resources.rs): `Overlay` enum, `Overlay::COUNT` const, `DebugOverlays` struct, and `flag_mut`/`is_active` methods are never used — forward-declared for future debug overlay system. Not errors.
- `unused_imports` warning (debug/resources.rs:3): `use bevy::prelude::*` is unused — visible in `dsclippy` (scenario runner build profile) and `dscheck`, not in `dclippy`. Appears as warning only. Added 2026-03-20.
- `must_use_candidate` error (scenario runner, breaker-scenario-runner/src/runner/execution.rs:139): `pub fn print_summary(results: &[(String, bool)]) -> i32` returns an exit code but lacks `#[must_use]`. Triggered by `-D clippy::pedantic` in the scenario runner crate. Fix: add `#[must_use]` attribute to `print_summary`. New as of 2026-03-20 — first time this error has appeared.

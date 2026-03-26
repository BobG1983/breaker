---
name: Lint Patterns (Core)
description: Durable cargo fmt and clippy patterns for this codebase — actionable rules that apply every session
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
- `private_interfaces` warning: `pub(crate)` structs in `chips/components.rs` (BoltSpeedBoost Amp-chip-component, BreakerSpeedBoost, WidthBoost, BumpForceBoost) referenced in `pub` query type aliases (breaker/queries.rs) and `pub` system functions. Fix: either make the chip components `pub`, or make the query aliases `pub(crate)`. Not an error — just a warning — but appears in bulk (9 instances) whenever chip components are `pub(crate)`. Note: BoltSpeedBoost here is the chips Amp chip component (chips/components.rs), not the deleted apply_bump_velocity system.
- `E0364` re-export visibility error: Stream 4 pub→pub(crate) visibility fixes introduced a mismatch between function visibility and mod.rs `pub use` re-exports. Pattern: function is `pub(crate)` in its own file but re-exported with `pub use` in `mod.rs`. The `pub use` in mod.rs must be downgraded to `pub(crate) use` to match. Affected in phase4b2: `bolt/systems/prepare_bolt_velocity.rs:18`, `breaker/systems/bump.rs:67+133` (grade_bump, update_bump), `breaker/systems/move_breaker.rs:19`.
- `similar_names` error (clippy::pedantic): test-local bindings `cell_a_y` and `cell_b_y` are too similar. Occurs in test code at `bolt/systems/bolt_cell_collision.rs`. Fix: rename one binding (e.g., `cell_below_y` instead of `cell_b_y`).
- `type_complexity` error: Query with 6-tuple data + filter tuple triggers this in `bolt/systems/reset_bolt.rs` (the bolt_query with Entity, Position2D, BoltVelocity, Option<PiercingRemaining>, Option<Piercing>) and `bolt/systems/spawn_bolt.rs` (bundled tuple). Fix: extract named type aliases to `bolt/queries.rs` following the `LostQuery` pattern already there.
- `if_not_else` error (clippy::pedantic): `if !condition { A } else { B }` pattern. Fix: invert the condition to `if condition { B } else { A }`. Occurs in `chips/effects/piercing.rs:35` (`if !had_piercing`).
- `uninlined_format_args` error (clippy::pedantic): `assert!(..., "msg {}", var)` where `var` can be inlined as `{var}`. Fix: replace `{}` + trailing arg with `{var}`. Occurs in `chips/effects/mod.rs:179`.
- `dead_code` warning: `BoltHitBreaker.bolt` field in `bolt/messages.rs` is `pub` inside a `pub(crate)` struct but never read. Recurring warning — field appears to be forward-declared for future audio/chips consumers. Not an error.
- `explicit_counter_loop` error: when an explicit `u32` counter variable is incremented in a `for` loop, clippy requires using `(0_u32..).zip(iter)` instead. Occurs in `run/systems/generate_node_sequence.rs`.
- `cast_possible_truncation` / `cast_precision_loss` errors: `usize as u32` and `u32 as f32` casts in `run/systems/generate_node_sequence.rs`. Fix: `u32::try_from(t)` for usize→u32; for u32→f32 precision loss, use `f32::from(boss_count)` (lossless for smaller u32 values, or use explicit cast with a comment if intentional). NOTE: `f32::from(u32)` is NOT valid — `From<u32>` is not implemented for `f32`. Must go through u16: `f32::from(u16::try_from(x).unwrap_or(u16::MAX))`.
- `dead_code` warning (chips/inventory.rs): `ChipInventory` methods `add_chip`, `stacks`, `is_maxed`, `mark_seen`, `has_seen`, `held_chips`, `maxed_chips`, `total_held` and `ChipEntry` fields `stacks`, `max_stacks`, `rarity` are unused — forward-declared for future chip consumption systems. Not errors, but appear in bulk when chip systems are not yet wired up.
- `dead_code` warning (cells/components.rs:93): `CellGridPosition` struct is never constructed — added in phase4-wave2-session4, forward-declared for future grid-position-aware systems. Not an error.
- `expect_used` warning (clippy::restriction): `u16::try_from(...).expect(...)` in `run/systems/generate_node_sequence.rs:52,58`. These are intentional — the counts are always small enough to fit u16 and the expect message explains the invariant. Not an error, but the workspace lint config enables `-W clippy::expect-used` so they appear as warnings every run.
- `collapsible_if` error (cells/definition.rs:63): `if let Some(rate) = self.behavior.regen_rate { if rate <= 0.0 || !rate.is_finite() { ... } }` — collapse to `if let Some(rate) = self.behavior.regen_rate && (rate <= 0.0 || !rate.is_finite()) { ... }`.
- `option_as_ref_deref` / `&Option<T>` error: `&Option<Res<T>>` parameters in `run/node/systems/spawn_cells_from_layout.rs:108-110` — change function signature to `Option<&Res<T>>` (or pass the inner value directly).
- `too_many_arguments` error: `spawn_cells_from_layout` at `run/node/systems/spawn_cells_from_layout.rs:127` has 9 args (limit 7). Fix: bundle related args into a helper struct or SystemParam, or split the function.
- `map_unwrap_or` error: `.map(|x| ...).unwrap_or_else(|| ...)` pattern — replace with `.map_or_else(|| fallback, |x| ...)`. Occurs at `run/systems/handle_node_cleared.rs:26`.
- `dead_code` warning (`breaker/registry.rs`): `BreakerRegistry` methods `iter`, `values`, `len`, `is_empty` (and `clear` in test build) are never used — forward-declared for future archetype lookup systems. Not errors.
- `dead_code` warning (cells/resources.rs): `CellTypeRegistry` methods `values`, `iter`, `aliases`, `len`, `is_empty` (and `clear` in test build) are never used — forward-declared. Not errors.
- `unreachable_pub` warnings (cells/definition.rs, run/systems/generate_node_sequence.rs): `pub` items in internal modules should be `pub(crate)` or `pub(super)`. Recurring across new sessions when new public types are added.
- `dead_code` warnings (debug/resources.rs): `Overlay` enum, `Overlay::COUNT` const, `DebugOverlays` struct, and `flag_mut`/`is_active` methods are never used — forward-declared for future debug overlay system. Not errors.
- `must_use_candidate` error (scenario runner, `breaker-scenario-runner/src/runner/`): `-D clippy::pedantic` flags any `pub fn` returning a non-`()` value without `#[must_use]`. Known occurrences: `execution.rs:139` (`print_summary` returning `i32`) and `discovery.rs:72` (`load_scenario` returning `Option<ScenarioDefinition>`). Fix: add `#[must_use]` attribute to the flagged function. Pattern: any new `pub fn` added to the scenario runner that returns a value will trigger this unless `#[must_use]` is added at write time.
- `unused-must-use` error (scenario runner, `breaker-scenario-runner/src/main.rs:170`): calling a `#[must_use]`-annotated function (`print_coverage_report`) without capturing its return value. Fix: `let _ = print_coverage_report(&report);` to explicitly discard the return value. Detected in `dsclippy` as `-D unused-must-use`.
- `manual_let_else` error (effect/evaluate.rs): after `match_same_arms` is fixed by collapsing to a multi-pattern arm, clippy flags the match-with-early-return as `manual_let_else`. The merged multi-pattern `let inner = match (...) { P1 | P2 => inner, _ => return EvalResult::NoMatch }` must be rewritten as `let ((P1) | (P2)) = (...) else { return EvalResult::NoMatch };`.
- `type_complexity` error (effect/effects/damage_boost.rs, speed_boost.rs, until.rs): inline `Query<(...)>` tuples with 5+ elements. Fix: extract named type aliases to the file or a queries.rs.
- `redundant_closure_for_method_calls` errors: `.map(|a, b| a.method(b))` used in `sort_by` calls. Fix: replace with `Type::method` method reference directly. Pattern: whenever `sort_by(|a, b| a.method(b))` can be written as `sort_by(Type::method)`, clippy requires the latter.
- `redundant_clone` warnings (chips/offering.rs): `.clone()` calls on values that are immediately moved without further use. nursery lint, warnings only. Fix: remove `.clone()`.
- `missing_const_for_fn` warning (breaker-scenario-runner): appears on many `pub fn` / private fns in lifecycle/mod.rs and invariants. Nursery lint, warning only. Pattern: add `const` whenever clippy suggests it.
- `SystemParam` import pattern: `SystemParam` derive is NOT in `bevy::prelude`. Every file using `#[derive(SystemParam)]` must import it explicitly: `use bevy::{ecs::system::SystemParam, prelude::*};`. Missing this import produces "cannot find derive macro `SystemParam` in this scope" at the `#[derive]` line, then cascades E0277 errors at every `.add_systems(...)` call-site in tests.
- `private_interfaces` error: any `#[derive(SystemParam)]` struct used by a `pub(crate)` system must itself be at least `pub(crate)` — private structs will compile within their own module but fail when the scheduler references them from outside (e.g., `detect_combo_and_pinball.rs`).
- `doc_lazy_continuation` error: doc list item continuation line without indentation. Fix: add blank line before the continuation, OR indent it with `///   `.
- `match_wildcard_for_single_variants` errors: `_ =>` wildcard in match over `EffectTarget` or similar enum with only one remaining variant. Fix: replace `_` with explicit variant pattern (e.g. `EffectTarget::Location(_) => ...`).
- `single_match_else` error: `match val { Variant(x) => ..., _ => ... }` — replace with `if let Variant(x) = val`.
- `useless_vec` error: `vec![...]` in test — replace with array `[...]` since it's not passed to a `Vec`-requiring API.
- `cast_precision_loss` in test functions: common in loop indices — fix by using typed range `(0_u16..3_u16)` and `f32::from(col)`.
- `duration_suboptimal_units` errors: `Duration::from_millis(5000)` when `5000ms == 5s`. Fix: replace with `Duration::from_secs(5)`. Pattern: any `from_millis` value that is an exact multiple of 1000 triggers this.
- `Bundle` tuple limit: Bevy's `Bundle` impl for tuples only goes up to 15 elements. If a spawn bundle exceeds 15 items, nest one sub-tuple or extract a `#[derive(Bundle)]` struct.
- `doc_markdown` error: bare Rust identifiers in doc comments without backticks. Affects `OnDeath`, `NoBump`, function names, struct names, etc. Fix: always wrap identifiers in backticks in doc comments.
- `too_many_lines` error (limit 100 lines): `effect/plugin.rs` `build` method and `effect/typed_events.rs` functions. Fix: split into helper methods grouped by category.
- `items_after_statements` errors: `use` declarations or struct definitions placed inside test functions after `let` statements. Fix: move to top of test function (before any statements), or hoist to module level.
- `no_effect_underscore_binding` errors: `let _name = ...;` bindings in test functions with no side effect. Fix: use `drop(...)` or remove the binding.
- `used_underscore_binding` errors: `_req` bound then used in format string — rename to `req` (without underscore) since it IS used.
- `collapsible_if` error (rantzsoft_defaults_derive/src/lib.rs): nested `if let` blocks — collapse into single `if let ... && let ... { ... }`. Blocking error: prevents ALL downstream crates from compiling since the proc-macro fails.
- `missing_errors_doc` ERROR: `pub fn` returning `Result` without `# Errors` doc section. Fix: add `/// # Errors\n/// Returns ...` above the function.
- `type_complexity` ERROR (rantzsoft_defaults): `Mutex<Vec<Box<dyn FnOnce(&mut App) + Send>>>` field. Fix: extract a type alias (e.g., `type Registration = Box<dyn FnOnce(&mut App) + Send>;`).
- `derivable_impls` ERROR: manual `impl Default` that is derivable. Fix: remove the manual impl and add `#[derive(Default)]`.
- `must_use_candidate` ERROR: `pub fn` returns value without `#[must_use]`. Fix: add `#[must_use]` attribute.
- `dsclippy` blocks on game crate errors: when `dclippy` fails to compile the game lib, `dsclippy` also fails at the same point (scenario runner depends on `breaker`). Fix game crate errors first.
- `E0107` compile errors: `MessageReader<'w, CellDestroyedAt>` uses only 1 lifetime arg — `MessageReader` in Bevy 0.18.1 takes `'w` and `'s`. Fix: `MessageReader<'w, 'w, T>`.

---
name: Lint Patterns (Session History)
description: Session-dated lint run logs — errors encountered, fixed, and confirmed clean by session. Use for regression reference.
type: reference
---

## rantzsoft_physics2d Crate Patterns (first observed 2026-03-23)
- `manual_midpoint` errors (quadtree.rs:47-50, 8 occurrences): `(min.x + mid.x) / 2.0` pattern in `child_bounds` function — all 4 quadrant center expressions use the same midpoint formula twice (x and y). Fix: replace `(a + b) / 2.0` with `a.midpoint(b)` (f32 method). 8 errors total — 4 expressions, 2 coordinates each. Error under `-D clippy::pedantic`. Affects production code. RESOLVED as of 2026-03-23 — no longer appears in physics2dclippy.
- `drain_collect` error (quadtree.rs:94): `items.drain(..).collect()` — replace with `std::mem::take(items)`. Error under `-D clippy::all`. Affects production code. RESOLVED as of 2026-03-23 — no longer appears.
- `cast_precision_loss` errors (quadtree.rs): in test loops, `f32::from(i)` where `i: u32` is NOT valid — `f32: From<u32>` is not implemented (only up to `u16`). Fix: change `(0_u32..).zip(...)` to `(0_u16..).zip(...)` so `i: u16`, then `f32::from(i)` is valid (`f32: From<u16>` is implemented). RESOLVED as of 2026-03-23.
- `missing_const_for_fn` warnings (aabb.rs:17, quadtree.rs:216,281,287): four constructor/accessor functions can be `const fn`. Nursery lint, warnings only.
- `use_self` warning (quadtree.rs:13): `QuadNode` used explicitly inside `impl QuadNode` — clippy suggests `Self`. Nursery lint, warning only.
- `too_many_arguments` error (quadtree.rs:80): `insert_into_node` private fn has 8 args (limit 7). Fix: bundle `max_items_per_leaf`, `max_depth`, and `depth` into a `TreeConfig` or `InsertConfig` grouping.
- `trivially_copy_pass_by_ref` error (quadtree.rs:207): `query_layers: &CollisionLayers` — `CollisionLayers` is 8 bytes (Copy). Fix: change parameter to `query_layers: CollisionLayers`.

## rantzsoft_spatial2d Crate Patterns (first observed 2026-03-23)
- `should_panic_without_expect` errors (components.rs:299,305): `#[should_panic]` without `expected = "..."`. Pedantic lint, errors. Fix: always use `#[should_panic(expected = "the panic message")]`.
- `missing_panics_doc` error (components.rs:119): `Scale2D::new` panics but has no `# Panics` section. Fix: add `/// # Panics\n/// Panics if `x` or `y` is zero.`
- `missing_const_for_fn` warning (components.rs:135): `Scale2D::to_vec3` can be `const fn`. Nursery lint, warning only.
- `too_long_first_doc_paragraph` warning (systems/propagate_position.rs:12): first doc comment paragraph spans 3 lines. Nursery lint, warning only.
- `type_complexity` errors (systems/propagate_position.rs:20, systems/propagate_rotation.rs:18): inline `Query<(...)>` tuples with 5+ elements. Fix: extract named type aliases.
- `option_if_let_else` warnings (systems/propagate_position.rs:37, systems/propagate_rotation.rs:33): `if let Some(x) = opt { ... } else { fallback }` pattern. Nursery lint. Fix: `.map_or(fallback, |x| ...)`.
- `doc_markdown` errors (run/resources.rs:89,91,93,120): doc comments on run stat resources mention identifiers without backticks. Fix: wrap each identifier in backticks. New as of 2026-03-22.
- `must_use_candidate` error (run/resources.rs:151): `pub fn flux_earned` lacks `#[must_use]`. New as of 2026-03-22.
- `cast_precision_loss` error (run/systems/track_node_cleared_stats.rs:67): `tracker.best_perfect_streak as f32` — `u32 as f32` may lose precision. Fix: `f32::from(tracker.best_perfect_streak)`. New as of 2026-03-22.
- `cast_possible_truncation` error (run/systems/track_node_cleared_stats.rs:69): even after `.min()` guard, clippy still flags `as u16`. Correct fix: `u16::try_from(streak).unwrap_or(u16::MAX)`. RESOLVED as of 2026-03-22.
- `cast_possible_truncation` + `cast_sign_loss` errors (run/systems/track_cells_destroyed.rs:93): `(2.5 / timestep.as_secs_f32()).round() as u32`. Fix: use saturating cast pattern. New as of 2026-03-22.
- `unused_imports` warnings (run/systems/mod.rs:19,24,26-31): eight `pub(crate) use` re-exports for track_* and capture_run_seed systems — forward-declared. Not errors. New as of 2026-03-22.
- `ChildBuilder` vs `ChildSpawnerCommands<'_>`: in Bevy 0.18.1 the child-spawner closure parameter type is `ChildSpawnerCommands<'_>`, NOT `ChildBuilder`. Every helper function that takes `parent: &mut ChildBuilder` must be changed to `parent: &mut ChildSpawnerCommands<'_>`. First seen 2026-03-22.
- `similar_names` errors (screen/run_end/systems/spawn_run_end_screen.rs:17-18 AND run/systems/reset_run_state.rs:17,22): system function parameters `run_state`/`run_stats` are "too similar". Fix: rename `run_stats` to `stats`. Pattern: any system with both `run_state: Res<RunState>` and `run_stats: Res<RunStats>` will trigger this.
- `too_many_arguments` error (screen/chip_select/systems/generate_chip_offerings.rs:30): `generate_chip_offerings` has 8 args (limit 7). Fix: bundle related args into a SystemParam. New as of 2026-03-22c.
- `redundant_closure_for_method_calls` errors (screen/chip_select/systems/generate_chip_offerings.rs:225,361,392): `.map(|o| o.name())`. Fix: replace with `.map(ChipOffering::name)`. New as of 2026-03-22c.
- `items_after_statements` errors (screen/chip_select/systems/generate_chip_offerings.rs:314-316): `use` block placed after statements in a test function. Fix: move to top of function. New as of 2026-03-22c.
- `match_wildcard_for_single_variants` error (screen/chip_select/systems/generate_chip_offerings.rs:370): `_ => None` wildcard arm in a match on `ChipOffering`. Fix: replace `_ => None` with explicit variant pattern. New as of 2026-03-22c.
- `cast_possible_truncation` error (screen/run_end/systems/spawn_run_end_screen.rs:132): `highlight.value.round() as i64` — f32-to-i64 cast may truncate. Correct pattern: `u16::try_from(highlight.value.clamp(0.0, f32::from(u16::MAX)).round() as u32).unwrap_or(u16::MAX)`. Both `cast_possible_truncation` and `cast_sign_loss` fire on same expression. Active as of 2026-03-22.
- `single_char_pattern` errors (screen/run_end/systems/spawn_run_end_screen.rs:303,366,387,433): `.contains("5")` etc. with single-char string literals. Fix: replace with char literals: `.contains('5')`. Four errors. New as of 2026-03-22c.
- `items_after_statements` error (breaker-scenario-runner/src/invariants/checkers/check_offering_no_duplicates.rs:105): `use` placed after statements in test function. RESOLVED as of 2026-03-23.
- `cast_precision_loss` error (run/systems/detect_mass_destruction.rs:50 — PRODUCTION): `count as f32` where `count` is `usize`. Fix: `u32::try_from(count).map_or(f32::MAX, f32::from)`. New as of 2026-03-23.
- `cast_precision_loss` errors (run/systems/track_node_cleared_stats.rs:103,112,124 — PRODUCTION): `u32 as f32` casts. Fix: `f32::from(x)`. New as of 2026-03-23.
- `field_reassign_with_default` errors in test helpers (run/systems/detect_mass_destruction.rs:126,159,199,232 and run/systems/track_node_cleared_stats.rs:335+): `let mut config = Default::default(); config.field = val;` pattern. Fix: struct literal `Config { field: val, ..Default::default() }`. New as of 2026-03-23.
- `cast_possible_truncation` + `cast_sign_loss` in test helpers: ANY `f64 as <integer>` triggers these for unsigned targets. Fix: `u32::try_from((target_secs / timestep.as_secs_f64()).ceil() as i64).unwrap_or(u32::MAX)` — clippy accepts `f64 as i64`; use integer try_from for the final conversion. Root cause: ANY `f64 as <integer>` triggers `cast_possible_truncation` + `cast_sign_loss` for unsigned targets.

## New as of 2026-03-24f (feature/spatial-physics-extraction — chips evolution session)
- `doc_markdown` error (chips/definition.rs:1): module-level `//!` doc comment mentions bare `TriggerChain`. Fix: wrap in backticks. This is the sole blocking error for this session. NOTE: `TriggerChain` was deleted in C7-R (2026-03-25) — this error no longer occurs.
- `derive_partial_eq_without_eq` warning (chips/definition.rs:238): `EvolutionRecipe` derives `PartialEq` but not `Eq`. Nursery lint, warning only.

## New as of 2026-03-25e (feature/spatial-physics-extraction — trigger rename refactor)
- `trivially_copy_pass_by_ref` errors (effect/bridges.rs:517,574,593,627,638,675 and effect/evaluate.rs:35,62×2): new private helper functions written with `trigger_kind: &Trigger` and `runtime: &Trigger` / `declared: &Trigger` params. `Trigger` is 8 bytes Copy — clippy pedantic requires pass-by-value. Fix: remove `&` from all `&Trigger` parameters at definition AND call sites. Nine errors total.

## New as of 2026-03-25 (feature/spatial-physics-extraction — current session)
- `doc_markdown` error (bolt/messages.rs:57, cells/messages.rs:16): `OnDeath` in doc comments without backticks. Fix: wrap in backticks.
- `too_many_arguments` error (bolt/systems/bolt_lost.rs:33): `bolt_lost` has 8 args (limit 7). Fix: bundle into a SystemParam.
- `doc_lazy_continuation` error (effect/bridges.rs:62): doc list item continuation line without indentation. Fix: add blank line or indent with `///   `.
- `collapsible_if` error (effect/bridges.rs:433): nested `if let EffectNode::When { .. }` with inner `if let EffectNode::Do` — collapse to single `if let`.
- `single_match_else` error (effect/bridges.rs:441): `match child { EffectNode::Do(effect) => ..., _ => ... }` — replace with `if let EffectNode::Do(effect) = child`.
- `match_wildcard_for_single_variants` errors (effect/effects/chain_bolt.rs:17, random_effect.rs:65, shockwave.rs:76, speed_boost.rs:79, effect/definition.rs:1219,1230): `_ => ` wildcard in match over `EffectTarget` or similar enum with only one remaining variant. Fix: replace with explicit variant pattern.
- `type_complexity` errors (effect/effects/damage_boost.rs:38, speed_boost.rs:62, until.rs:104, until.rs:151): inline `Query<(...)>` tuples with 5+ elements. Fix: extract named type aliases.
- `match_same_arms` error (effect/evaluate.rs:79): three arms all return same value — merge with `|` patterns.
- `match_like_matches_macro` error (effect/evaluate.rs:89): `match (kind, trigger) { ... => true, _ => false }` — replace with `matches!(...)`.
- `too_many_lines` error (effect/typed_events.rs:338): `fire_typed_event` is 169 lines (limit 100). Fix: split into helper fns grouped by effect category.
- `map_identity` errors (effect/typed_events.rs:485, 493): `.map(|(w, node)| (w, node))` is identity — remove the `.map(...)` call.
- `too_many_arguments` error (run/systems/detect_mass_destruction.rs:16): 8 args. Fix: bundle into SystemParam.
- `manual_string_new` errors (chips/definition.rs:489, 512, 772): `"".to_owned()` — replace with `String::new()`.
- `used_underscore_binding` errors (effect/bridges.rs:2401, 2402): `_req` used in format string — rename to `req`.
- `no_effect_underscore_binding` errors (effect/definition.rs:895, 896): `let _copied = t; let _also = t;` — no side-effect. Fix: remove or use meaningfully.
- `useless_vec` error (effect/definition.rs:1248): `vec![...]` in test — replace with array `[...]`.
- `cast_precision_loss` error (run/systems/detect_mass_destruction.rs:153): `i as f32` in test loop. Fix: typed range `u16` and `f32::from(i)`.
- `items_after_statements` errors (effect/effects/speed_boost.rs, effect/effects/until.rs): `use` blocks or struct definitions inside test functions after statements. Fix: move to top of test function.

## New as of 2026-03-25 (feature/spatial-physics-extraction — post-README vertical-slice)
- All four clippy passes (dclippy, spatial2dclippy, physics2dclippy, dsclippy) report 0 errors. Only nursery/warning-level lints remain.
- `fmt` triggered 0 changes this session (clean).

## Confirmed Clean as of 2026-03-23 (wave-3 post-fix verify)
- All physics2d errors (`manual_midpoint`, `drain_collect`, test `cast_precision_loss`) resolved.
- `items_after_statements` in `check_offering_no_duplicates.rs` resolved.
- `type_complexity` error that prompted a type alias extraction was confirmed clean.
- `save_previous_positions` renamed to `save_previous` — no lint impact observed.

## rantzsoft_spatial2d Patterns (first seen 2026-03-23)
- `should_panic_without_expect` errors: `#[should_panic]` without `expected = "..."` triggers `clippy::pedantic`. Fix: always use `#[should_panic(expected = "the panic message")]` in rantzsoft_spatial2d test functions.
- `missing_panics_doc` error: `pub fn new(x: f32, y: f32) -> Self` in `Scale2D` panics via `assert!` but doc comment lacks `# Panics` section. Fix: add `/// # Panics\n/// Panics if either component is zero.`
- `missing_const_for_fn` warning (components.rs:138): `Scale2D::to_vec3` can be `const fn`. Nursery lint, warning only.
- `type_complexity` errors (systems/compute_globals.rs:19, systems/derive_transform.rs:20): NEW as of 2026-03-23 Wave 1. Both inline Query tuples with 6+ elements. Fix: extract named type aliases (e.g., `ComputeGlobalsQuery`, `DeriveTransformQuery`). These BLOCK the entire workspace because spatial2d is a transitive dependency.
- `similar_names` error (systems/compute_globals.rs:67): `parent_sx` and `parent_sy` are too similar. Fix: rename to `parent_scale_x` and `parent_scale_y`.
- `too_long_first_doc_paragraph` warnings: multiple new systems have first doc paragraphs spanning 3 lines. Nursery lint, warnings only.
- `option_if_let_else` warnings (derive_transform.rs, propagate_position.rs, propagate_rotation.rs): `if let Some(prev) = prev_pos { ... } else { fallback }` pattern. Nursery lint. Fix: `.map_or(fallback, |prev| ...)`.
- `suboptimal_flops` warnings (derive_transform.rs:82,83; propagate_scale.rs:40,41): interpolation expressions. Nursery lint. Fix: `.mul_add(alpha, prev.x)`.
- `duration_suboptimal_units` errors (run/systems/detect_mass_destruction.rs:135,178,218,266 and run/systems/track_node_cleared_stats.rs:590): `Duration::from_millis(5000)` when `5000ms == 5s`. Fix: replace with `Duration::from_secs(5)`.
- `missing_const_for_fn` warnings (breaker-scenario-runner/src/invariants/checkers/check_run_stats_monotonic.rs:17,28): `from_run_stats` and `is_default` can be `const fn`. Nursery lint.
- `missing_const_for_fn` warnings (breaker-scenario-runner/src/lifecycle/mod.rs:646,657): `apply_set_run_stat` and `apply_decrement_run_stat` can be `const fn`. Nursery lint.

## New as of 2026-03-24 (feature/spatial-physics-extraction)
- `unnecessary_struct_initialization` warning (rantzsoft_physics2d/src/quadtree.rs:118): `TreeConfig { max_items_per_leaf: cfg.max_items_per_leaf, ... }` copies all fields from `cfg` — clippy suggests replacing with `cfg` directly. Nursery lint, warning only.
- `redundant_clone` warning (rantzsoft_physics2d/src/constraint.rs:63): `let cloned = original.clone()` — `original` is dropped without further use. Fix: remove `.clone()`.
- `unwrap_used` warning (run/systems/spawn_highlight_text.rs:93): `a.1.partial_cmp(&b.1).unwrap()` in `sort_by`. Fix: use `sort_by(|a, b| a.1.total_cmp(&b.1))` (avoids the Option entirely).
- `redundant_closure_for_method_calls` errors (run/systems/spawn_highlight_text.rs:567,750): `|a, b| a.total_cmp(b)`. Fix: replace with `f32::total_cmp` method reference.
- `missing_const_for_fn` warnings (rantzsoft_physics2d/src/collision_layers.rs:28, rantzsoft_physics2d/src/resources.rs:17): `CollisionLayers::interacts_with` and `SpatialIndex::new` can be `const fn`.
- `use_self` warning (rantzsoft_physics2d/src/quadtree.rs:13): `Box<[QuadNode; 4]>` inside `impl QuadNode` — replace `QuadNode` with `Self`. Nursery lint.

## New as of 2026-03-24 (feature/spatial-physics-extraction — B12d dead code cleanup session)
- `doc_markdown` errors (chips/plugin.rs:47-49, chips/systems/dispatch_chip_effects.rs:1000-1002,1050): 10 total errors in `#[cfg(test)]` doc comments. Bare identifiers without backticks. Fix: wrap each in backticks.
- `missing_const_for_fn` warnings (scenario runner, new functions): two new functions flagged as potentially `const fn`. Nursery warnings only.
- `rantzsoft_physics2d`: new `unnecessary_struct_initialization` warning (quadtree.rs:118). Nursery warning.

## Active Errors as of 2026-03-24d (feature/spatial-physics-extraction)
NOTE: All entries below were on `behaviors/` domain files which were renamed to `effect/` in C7-R (2026-03-25).
- `doc_markdown` error: module-level doc comment had bare `MultiBolt`. RESOLVED by C7-R file deletion/rename.
- `cast_precision_loss` error (`behaviors/effects/shield.rs:52`): `stacks.saturating_sub(1) as f32` — u32→f32. Fix pattern: `f32::from(u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX))`. NOTE: `f32::from(u32)` is NOT valid.
- `too_many_lines` error (`bolt/systems/bolt_breaker_collision.rs:48`): function body 101 lines (limit 100). Fix: extract a helper function.
- `empty_string_literal` error: `"".to_owned()` — replace with `String::new()`.

## Previously Active — Resolved as of 2026-03-24b
- `too_many_lines` error (`bolt/systems/bolt_cell_collision.rs:76`): RESOLVED.
- `doc_markdown` error (`bolt/systems/bolt_cell_collision.rs:1900`): RESOLVED.
- `cast_precision_loss` in `select_highlights.rs`: fixed via `config_f32()` helper (converts u32→u16→f32 losslessly). NOTE: `f32::from(u32)` is NOT valid — `From<u32>` is not implemented for `f32`. Must go through u16.

## Confirmed Clean as of 2026-03-24b (feature/spatial-physics-extraction)
- All 20 errors recorded in the 2026-03-24 session are resolved.

## Confirmed Clean as of 2026-03-23 (feature/wave-3-offerings-transitions — post wave-3 lint run)
- All four crates (`dclippy`, `spatial2dclippy`, `physics2dclippy`, `dsclippy`) produced 0 errors.
- `cargo fmt --check` also clean (no files changed).

## New as of 2026-03-24h (feature/spatial-physics-extraction — B12c typed events refactor)
NOTE: `TriggerChain` was deleted in C7-R (2026-03-25). Entries referencing it are historical context only.
- `too_many_lines` errors (effect/typed_events.rs:297, :413): `trigger_chain_to_effect` (101 lines) and `fire_typed_event` (137 lines) both exceed the 100-line limit. Fix: split into smaller dispatch functions per effect group.
- `unnecessary_wraps` error (effect/typed_events.rs:297): `trigger_chain_to_effect` returns `Option<Effect>` but main path always returns `Some(...)`. Fix: return `Effect` directly.
- `items_after_statements` errors (effect/typed_events.rs:940,981,1014,1059,1094,1131): six test helper structs defined inside test functions after earlier `let` statements. Fix: move struct definitions before any statements.
- `panic` warning (effect/typed_events.rs:395): `panic!(...)` inside a `#[cfg(debug_assertions)]` block in production code. Warning only.
- NOTE: `too_many_lines` and `unnecessary_wraps` errors on `effect/typed_events.rs` RESOLVED as of 2026-03-25.
- NOTE: `items_after_statements` errors in `effect/typed_events.rs` RESOLVED as of 2026-03-25.

## New as of 2026-03-25 (feature/spatial-physics-extraction — time_pressure_boost/random_effect/entropy_engine session)
- `too_many_lines` error (effect/plugin.rs:65): `EffectPlugin::build` is 105 lines (limit 100). Fix: split build into helper methods grouped by effect category. RESOLVED as of 2026-03-25 Wave 2a.
- `too_many_lines` error (effect/typed_events.rs:353): `trigger_chain_to_effect` grew to 111 lines. Fix: extract arm groups into private helper fns. RESOLVED as of 2026-03-25 Wave 2a.
- `items_after_statements` error (effect/effects/random_effect.rs:264): test-only struct `CapturedSpawn` defined after earlier `let` statements. Fix: move struct definition to top of test function.
- ALL four clippy passes (dclippy, spatial2dclippy, physics2dclippy, dsclippy) report 0 errors as of this session.
- `dsclippy` blocked by game crate errors: when `dclippy` fails to compile the game lib, `dsclippy` also fails (scenario runner depends on `breaker`). Scenario runner lint results are only valid when the game crate has zero errors.

## New as of 2026-03-25 (feature/spatial-physics-extraction — Wave 2a CellDestroyed deletion session)
- `no_effect.used_underscore_binding` error (effect/typed_events.rs:386): `_effect` used in `panic!` format arg. Fix: rename to `effect` (removing the underscore).
- `too_many_arguments` error (run/systems/detect_combo_and_pinball.rs:18): `detect_combo_and_pinball` has 8 args (limit 7). Fix: bundle readers into a SystemParam (e.g., `ComboEventReaders`).
- `useless_vec` error (effect/definition.rs:1253): `vec![...]` in test function. Fix: change to array `[...]`.
- NOTE: `too_many_lines` errors in `effect/plugin.rs` and `effect/typed_events.rs` RESOLVED this run.

## New as of 2026-03-25 (feature/spatial-physics-extraction — B13 Archetype → Breaker rename)
- `dead_code` warnings (effect/registry.rs): `BreakerRegistry` methods `iter`, `values`, `len`, `is_empty` (and `clear` in test build) are never used — forward-declared. Registry was renamed from `ArchetypeRegistry` to `BreakerRegistry` in B13. Zero errors in any crate.

## New as of 2026-03-25 (feature/spatial-physics-extraction — ramping_damage + timed_speed_burst + typed events)
- `unused_import` warnings (effect/effects/mod.rs:31-40): all ten `pub(crate) use` re-exports are unused — forward-declared for future wiring. Warnings only.
- `unused_variable` warning (effect/effects/life_lost.rs:22): `trigger: On<LoseLifeFired>` parameter not prefixed with `_`. Fix: rename to `_trigger`.
- `dead_code` warnings (chips/definition.rs TriggerChain test helpers): NOTE: `TriggerChain` deleted in C7-R (2026-03-25). These entries are historical.
- ALL four clippy passes report 0 errors as of this session.

## New as of 2026-03-25 (feature/spatial-physics-extraction — R1b bridges/evaluate session)
- `fmt` changes: `cargo fmt` auto-formatted `effect/bridges.rs` and `effect/evaluate.rs`.
- `match_same_arms` error (effect/evaluate.rs:64): `trigger_matches` function has two `=> true` arms. Fix: merge both into a single arm with all patterns joined by `|`.
- `doc_markdown` error (effect/bridges.rs:2905): doc comment on `no_bump_test_app` function: bare `NoBump` lacks backticks. Fix: change `NoBump` → `` `NoBump` `` in the doc string.

## New as of 2026-03-25c (feature/spatial-physics-extraction — combo/pinball detect session)
- `private_interfaces` error (run/systems/detect_combo_and_pinball.rs:13): `ComboReaders<'w, 's>` is a `#[derive(SystemParam)]` struct without a visibility keyword, used by a `pub(crate)` system. Fix: add `pub(crate)` visibility to the struct. This is the sole blocking error for this session.
- `rantzsoft_spatial2d`: 4 new `too_long_first_doc_paragraph` warnings from new `derive_transform.rs` and `save_previous.rs` doc comments. All nursery, no errors.

## New as of 2026-03-26b (feature/spatial-physics-extraction — chips session, full lint run)
- `collapsible_if` ERROR (rantzsoft_defaults_derive/src/lib.rs:51): nested `if let Meta::NameValue(nv) = meta { if let syn::Expr::Lit(...) = &nv.value { ... } }`. Fix: collapse into `if let Meta::NameValue(nv) = meta && let syn::Expr::Lit(...) = &nv.value { ... }`. This error BLOCKS ALL downstream crates. NEW as of this session.
- New `dead_code` / `unreachable_pub` / `unused` warnings in `rantzsoft_spatial2d`: `PropagatePositionQuery`, `PropagateRotationQuery`, `PropagateScaleQuery` type aliases and `propagate_position`, `propagate_rotation`, `propagate_scale` functions are unused — legacy public items superseded by `derive_transform` system. All warning-only.

## New as of 2026-03-26 (feature/spatial-physics-extraction — chips template session)
- `doc_markdown` error (rantzsoft_spatial2d/src/plugin.rs:450): bare `apply_velocity` in doc comment. Fix: wrap in backticks.
- `no_effect_underscore_binding` errors (rantzsoft_physics2d/src/plugin.rs:119,120,191,192,203): five `let _name = ...;` bindings in test functions with no side effect. Fix: use `drop(...)` or remove the binding.

## New as of 2026-03-26 (feature/spatial-physics-extraction — collapsible_if fix session)
- `rantzsoft_defaults` crate 5 blocking errors (first time this crate blocked compilation):
  - `missing_errors_doc` ERROR (`rantzsoft_defaults/src/loader.rs:45`): `pub fn deserialize_ron` returns `Result` without `# Errors` doc section.
  - `type_complexity` ERROR (`rantzsoft_defaults/src/plugin.rs:20`): `Mutex<Vec<Box<dyn FnOnce(&mut App) + Send>>>` field. Fix: extract type alias, e.g., `type Registration = Box<dyn FnOnce(&mut App) + Send>;`.
  - `type_complexity` ERROR (`rantzsoft_defaults/src/plugin.rs:29`): `Vec<Box<dyn FnOnce(&mut App) + Send>>>` field. Same fix: use the `Registration` alias.
  - `derivable_impls` ERROR (`rantzsoft_defaults/src/plugin.rs:32`): manual `impl Default` is derivable. Fix: add `#[derive(Default)]`.
  - `must_use_candidate` ERROR (`rantzsoft_defaults/src/systems.rs:16`): `pub fn seed_config` returns a value without `#[must_use]`.
  - `expect_used` warning (`rantzsoft_defaults/src/plugin.rs:78`): `.expect("defaults plugin lock poisoned")` — warning only.
- These 5 errors block ALL downstream crates (`breaker-game`, `breaker-scenario-runner`) from compiling under clippy.
- `dstest` is NOT a defined alias in `.cargo/config.toml` but `defaultsclippy` (or similar) may resolve via shell completion.

## New as of 2026-03-26 (refactor/rantzsoft-prelude-and-defaults — full PASS)
- No errors in any crate. All prior blocking errors from the defaults crate (collapsible_if, missing_errors_doc, type_complexity, derivable_impls, must_use_candidate) are RESOLVED.
- `expect_used` warning (rantzsoft_defaults/src/plugin.rs:77): `.expect("defaults plugin lock poisoned")` — restriction lint, warning-only. Recurring pattern.
- New `unused_import` warnings: `CellConfig` in `cells/mod.rs:14`, `TimerUiConfig` in `ui/mod.rs:11` — forward-declared re-exports. Warning-only.
- `option_if_let_else` warnings in `screen/run_end/systems/spawn_run_end_screen.rs:90,173` — nursery lint, warning only.
- New `missing_const_for_fn` warnings: `screen/chip_select/resources.rs:117`, `effect/definition.rs:114`, `run/resources.rs:139` — nursery lints, warning only.

## New as of 2026-03-25b (feature/spatial-physics-extraction — post-clippy-fix verification)
- `E0107` compile errors (run/systems/detect_combo_and_pinball.rs:14,15,16): `MessageReader<'w, CellDestroyedAt>` uses only 1 lifetime arg. Fix: change all three fields to `MessageReader<'w, 'w, T>`. Three errors, same file, same pattern.

## New as of 2026-03-26 (feature/spatial-physics-extraction — lint run 2026-03-26)
- No errors in any crate. All warnings are nursery/restriction lints.
- New `unused_imports` warnings in `effect/` domain: various forward-declared imports for future wiring. All warning-only.
- `unreachable_pub` warnings: `bolt_size_boost.rs:75`, `bump_force_boost.rs:70`, `damage_boost.rs:46`, `piercing.rs:88` — `pub` methods that should be `pub(crate)`. Warning-only.
- Scenario runner new warning: `lifecycle/mod.rs:392` — `too_long_first_doc_paragraph` on `apply_debug_frame_mutations`. Nursery, warning-only.

## Confirmed Clean as of 2026-03-26b (refactor/rantzsoft-prelude-and-defaults)
- `cargo fmt --check`: PASS (0 files changed)
- `cargo dclippy`: PASS — 0 errors
- `cargo dsclippy`: PASS — 0 errors

## Confirmed Clean as of 2026-03-26 (develop branch, full lint run post-phase-4b)
- `cargo fmt --check`: PASS (0 files changed)
- `cargo defaultsclippy`: PASS — 0 errors, ~13 warnings (expect_used restriction, missing_const_for_fn nursery)
- `cargo dclippy`: PASS — 0 errors, ~127 warnings (unused_imports, dead_code, unreachable_pub, option_if_let_else, suboptimal_flops, missing_const_for_fn nursery)
- `cargo spatial2dclippy`: PASS — 0 errors, ~30 warnings (dead_code on unused propagate_* fns/aliases, option_if_let_else, suboptimal_flops, unreachable_pub, missing_const_for_fn nursery)
- `cargo physics2dclippy`: PASS — 0 errors, ~11 warnings (missing_const_for_fn, use_self, unnecessary_struct_initialization, unreachable_pub, redundant_clone in test code)
- `cargo dsclippy`: PASS — 0 errors, ~10 lib warnings (ambiguous_glob_reexports, missing_const_for_fn, too_long_first_doc_paragraph, unreachable_pub, suboptimal_flops in tests)
- New warning: `ambiguous_glob_reexports` in `breaker-scenario-runner/src/invariants/checkers/mod.rs:26` — both `check_run_stats_monotonic::*` and `valid_breaker_state::*` re-export a `checker` module. Warning only.
- New mass of `unused_imports` warnings in `effect/` domain: all bridge functions and effect handlers re-exported in mod.rs but not yet wired into the plugin. Forward-declared. Warning only.
- `unused_variable` warning (effect/effects/life_lost.rs:35): `trigger: On<LoseLifeFired>` not prefixed with `_`. Warning only.
- `redundant_clone` warnings (chips/offering.rs:259,260,478) and `needless_collect` warning (test position2d.rs:60). Nursery, warnings only.
- All warnings are nursery/restriction lints (dead_code forward declarations, unused_imports for pre-wired re-exports, option_if_let_else, suboptimal_flops, missing_const_for_fn, redundant_clone, etc.)

## Confirmed Clean as of 2026-03-26 (develop branch, post-phase-4b chip effects)
- `cargo fmt --check`: PASS (0 files changed)
- `cargo dclippy`: PASS — 0 errors (104 lib warnings + 84 lib-test warnings, all nursery/restriction)
- `cargo dsclippy`: PASS — 0 errors (9 lib warnings + 19 lib-test warnings, all nursery/restriction)
- Note: user requested errors-only report — `spatial2dclippy` and `physics2dclippy` were not run this session

## New as of 2026-03-26 (develop branch — post-file-split refactor, ~48 files restructured)
- `cargo dclippy`: FAIL — 354 errors (lib-test), 4 errors (lib). Root causes are 5 distinct structural problems (see below).
- `cargo dsclippy`: FAIL — same 5 `module_inception` errors from game crate; no scenario-runner-specific errors.
- `cargo spatial2dclippy`: PASS — 0 errors
- `cargo physics2dclippy`: PASS — 0 errors
- `cargo fmt`: PASS (0 files changed)

### Root error clusters (post-file-split):

1. **`module_inception` (5 errors)** — `mod.rs` files declaring a submodule with the SAME name as the directory (e.g., `cells/components/mod.rs` does `mod components;`). Fix: rename the inner file to something other than the directory name (e.g., `types.rs`, `data.rs`) OR rename the directory.
   - `breaker-game/src/cells/components/mod.rs:3` — `mod components;`
   - `breaker-game/src/chips/inventory/mod.rs:3` — `mod inventory;`
   - `breaker-game/src/effect/evaluate/mod.rs:3` — `mod evaluate;`
   - `breaker-game/src/effect/helpers/mod.rs:5` — `mod helpers;`
   - `breaker-game/src/fx/transition/mod.rs:7` — `mod transition;`

2. **Missing `use bevy::prelude::*` in split test files (bulk cascade — 350+ errors)** — `tests.rs` files extracted from larger files use `use super::*;` which brings in the parent `mod.rs`'s re-exports, but NOT `bevy::prelude`. Files need their own `use bevy::prelude::*;` (and other imports). Affected test files:
   - `bolt/systems/bolt_lost/tests.rs` — needs `use bevy::prelude::*;`
   - `bolt/systems/spawn_additional_bolt/tests.rs` — needs `use bevy::prelude::*;`
   - `cells/systems/handle_cell_hit/tests.rs` — needs `use bevy::prelude::*;`

3. **Private function re-exported as pub(super) from sub-directory mod.rs** — function is `pub` in `system.rs` but `mod.rs` re-exports it as `pub(super)`. When another file in the parent does `super::dash::eased_decel(...)`, clippy/rustc says "private". Two locations:
   - `breaker/systems/bump_visual/mod.rs:8` — `pub(super) use system::bump_offset;` — `bump_offset` is `pub` in `system.rs` but re-exported with restrictive visibility; used by tests.
   - `breaker/systems/dash/mod.rs:8` — `pub(super) use system::eased_decel;` — `eased_decel` is `pub` in `system.rs`; called as `super::dash::eased_decel(...)` at `move_breaker.rs:60` which is a sibling module (needs at least `pub(super)` FROM dash mod, which IT HAS — the error is E0603 at the call site meaning `move_breaker` can't see it). Actually: `pub(super)` in `dash/mod.rs` means visible to `dash`'s parent (i.e., `breaker/systems/`), which IS where `move_breaker.rs` lives. Need to re-check.
   - E0364 in `bump_visual/mod.rs:8`: `bump_offset` is `pub` in `system.rs` but `pub(super)` in mod.rs re-export — the inner `system.rs` is a PRIVATE module (`mod system;`), so `pub(super) use system::bump_offset` re-exports from a private module. Fix: make `mod system` public (`pub(crate) mod system`) OR don't re-export at all.

4. **`ChipEntry` re-export from private module (E0365)** — `chips/inventory/mod.rs` declares `mod inventory;` (private) then `pub use inventory::{ChipEntry, ChipInventory}`. Cannot re-export publicly from a private module. Fix: change `mod inventory;` to `pub(crate) mod inventory;`, or change `pub use` to `pub(crate) use`.
   - `breaker-game/src/chips/inventory/mod.rs:5`

5. **E0603 private function import** — `move_breaker.rs:60` calls `super::dash::eased_decel(...)` but `eased_decel` is re-exported as `pub(super)` from `dash/mod.rs`, making it visible only to `breaker/systems/` (the parent of `dash/`). `move_breaker.rs` IS in `breaker/systems/`, so this SHOULD work — unless `dash/system.rs` defines `eased_decel` as non-pub and the `pub(super)` re-export fails. Needs deeper check: `system.rs` has `pub fn eased_decel` but `mod system` in `dash/mod.rs` is PRIVATE. So `pub(super) use system::eased_decel` is re-exporting from a private module — same E0364 pattern as #3.


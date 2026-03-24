---
name: Intentional Patterns & Vocabulary
description: Patterns that look wrong but are correct, plus vocabulary decisions
type: reference
---

## Intentional Patterns (Do Not Flag)
- `existing.iter().next().is_some()` in `spawn_lives_display` — minor inconsistency; prefer `!is_empty()` in new code.
- `spawn_side_panels` uses `!existing.is_empty()` — preferred form.
- `collect_scenarios_recursive` uses `&mut Vec<PathBuf>` out-parameter — intentional for recursive DFS.
- `let _ = &defaults;` in `apply_archetype_config_overrides` — intentional placeholder.
- Heavy `.unwrap()` in test code only — all production paths use fallible patterns.
- `(ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>)` tuple system param in `spawn_bolt` and `spawn_cells_from_layout` — intentional Bevy workaround for multiple `ResMut` borrows from the same world; Bevy disallows two separate `ResMut<T>` params for distinct types in the same system in some versions. Do not flag.
- `#[cfg(all(test, not(target_os = "macos")))]` on integration tests — platform guard.
- `#[allow(dead_code)]` on BumpPerformed and CellDestroyed — message type derive macro limitation. Intentional.
- Double-insert in `init_breaker_params` — Bevy 15-component tuple limit workaround.
- `handle_cell_hit.rs` `peekable().peek().is_none()` early-return — this pattern has been removed in the current codebase; `handle_cell_hit` now uses a `despawned: Local<Vec<Entity>>` guard and iterates normally. No longer present. CLOSED as of 2026-03-19.
- `scenario_actions.len() as u32` in lifecycle.rs — safe in practice.

- `StressFailure` / `StressResult` / `copy_index` in execution.rs — runner-internal infrastructure terms; no game vocabulary rule applies to the scenario runner's own tooling types.
- `stress_copy` flag in main.rs — internal CLI flag name for subprocess guard; not a game vocabulary term.

- `make_layout` helper in `apply_entity_scale_to_breaker.rs` tests returns `NodeLayout` (caller wraps in `ActiveNodeLayout`); `make_layout` in `apply_entity_scale_to_bolt.rs` tests returns `ActiveNodeLayout` directly. Intentional asymmetry in helper return types — both are correct. Do not flag.
- `_entity_scale` binding (bolt_lost.rs map closure): intentionally named-ignored because the filter closure already consumed the scale value. Do not flag as unused.
- `LostBoltEntry::is_extra: bool` — two-state field (ExtraBolt or not), no third state, acceptable per established SeedEntry::focused pattern.

- `SendBoltLostFlag(bool)` in bridges.rs tests — inconsistent with all other `Send*(Option<T>)` test helpers in the same file. Flag as a style inconsistency (should be `Option<BoltLost>`).
- `pub enum ImpactTarget` / `pub enum TriggerChain` in definition.rs — `pub` (not `pub(crate)`) is justified: `chips/mod.rs` re-exports them as `pub use`, and the scenario runner crate uses `breaker::chips::TriggerChain` directly in `types/mod.rs:272`. Do not flag.
- `armed_query: Query<(Entity, &mut ArmedTriggers)>` (no `mut` on binding) in `bridge_cell_destroyed` and `bridge_bolt_lost` — correct; the `mut` is inside the query type for `ArmedTriggers`. Binding mutability not needed because `evaluate_armed_all` takes the query by value (moves it). Do not flag.

## Vocabulary Decisions
- `format_lives` in `life_lost.rs` — "lives" is correct game vocabulary (count of `LivesCount`).
- `fire_consequences` in `bridges.rs` — "consequence" used in its precise game-system sense.
- `upgrade` module/type names — infrastructure wrappers around Amp/Augment/Overclock; acceptable.
- `ChaosDriver` — renamed from `ChaosMonkey` in feature/scenario-coverage-expansion. Rename is complete in production code (`src/input.rs`). Test bodies still use `monkey` as local variable names (`let mut monkey = ChaosDriver::new(...)`) — acceptable in test-only code.
- `HybridInput` scripted phase boundary: doc says `0..scripted_frames` exclusive, implementation uses `frame < scripted_frames` (correct). The edge-case test probes frame 99 (not frame 100); comment in test says "last scripted frame". This is correct — 99 is inside scripted phase when scripted_frames=100.
- `seed_archetype_registry` test fixture previously used `make_archetype("Flux")` — renamed to `make_archetype("Vortex")` in a subsequent PR. CLOSED as of 2026-03-19 full-codebase review.
- `SeedEntry::value: String` field named `value` — flagged as vague in isolation but this is a UI resource where `value` is the canonical name for the text field's contents (mirroring HTML input semantics). Acceptable.
- `SeedEntry::focused: bool` — acceptable; the alternative `is_focused` would be misread as a method name on the struct. The `bool` field for a two-state focus condition is not a candidate for an enum since there is no third state.
- `stack_u32` / `stack_f32` private helpers — these are module-private (no `pub`), so `value` / `per_stack` parameter names are acceptable.
- `apply_chip_effect` uses `use crate::chips::components::*` (glob import) — 9 types from the same path; this matches the project's 4+ items → glob rule. Intentional.
- `PendingChipSelected` test-only resource in `apply_chip_effect.rs` — test infrastructure pattern, identical to `PendingChipSelected` used elsewhere. Do not flag.
- `ArchetypeRegistry` exposes `archetypes: HashMap<String, ArchetypeDefinition>` as a `pub` field — does NOT follow the encapsulated registry pattern used by ChipRegistry and NodeLayoutRegistry. This is a pre-existing divergence, not new to the cleanup branch. data.md says "sorted by name for UI display" but the field is just a raw HashMap (sorting happens at the call site in handle_run_setup_input). Do not flag as a new issue.
- `AugmentEffect::BumpForce` variant name vs `BumpForceBoost` component name — naming asymmetry is intentional: the enum variant describes the RON-facing effect name, the component adds the "Boost" suffix for clarity. Same for `TiltControl` vs `TiltControlBoost`. Acceptable.
- Per-effect observer files (`bolt_speed_boost.rs`, `chain_hit.rs`, `bolt_size_boost.rs`, `breaker_speed_boost.rs`, `bump_force_boost.rs`, `tilt_control_boost.rs`) now have stacking + cap tests as of feature/phase4b2-effect-consumption. This gap is CLOSED.
- `NodeType` enum in `difficulty.rs` — uses `Passive`/`Active`/`Boss` not game vocabulary (`Passive`/`Active`/`Boss` are structural names for the difficulty curve, not player-facing terms; acceptable).
- `NodePool` enum in `definition.rs` — same reasoning as NodeType; `Passive`/`Active`/`Boss` are pool tags, not game vocabulary violations.
- `run/definition.rs` vs `run/resources.rs` split — `definition.rs` holds RON-deserialized content types (`NodeType`, `TierNodeCount`, `TierDefinition`, `DifficultyCurveDefaults`); `resources.rs` holds Bevy `Resource` types and runtime state (`RunState`, `RunOutcome`, `NodeSequence`, `DifficultyCurve`). This is the established split; do not flag.
- `handle_run_lost` and `handle_timer_expired` are `pub` (not `pub(crate)`) in `run/systems/mod.rs` — they are registered only inside `RunPlugin` (same crate), so `pub(crate)` would suffice. Pre-existing pattern; acceptable until a cleanup pass targets visibility across the run domain.
- `bolt/systems/mod.rs`: `spawn_bolt_lost_text`, `hover_bolt`, `init_bolt_params`, `launch_bolt`, `spawn_additional_bolt` are `pub` but only consumed by `BoltPlugin` inside the same crate. Pre-existing pattern matching run-domain behavior. Do not flag as a new issue.
- `CellSpawnContext` SystemParam in `spawn_cells_from_layout.rs` is `pub(crate)` — used only inside the node subdomain and by `spawn_cells_from_grid` (dev feature). Correct visibility.
- `reader.read().count() == 0` early-exit in `bridge_cell_destroyed` and `bridge_bolt_lost` — consumes the message iterator entirely just to check for presence; the idiomatic alternative (`reader.is_empty()` or `peekable`) may not be available depending on the MessageReader API. Do not flag unless the API supports a non-consuming peek.
- `_entity` in `for (_entity, mut armed) in &mut armed_query` in `evaluate_armed_all` — entity is destructured but not used because the function fires/re-arms based on chain state alone (CellDestroyed/BoltLost are global events). Acceptable.
- `shockwave_no_op_when_bolt_despawned` test creates two separate `App` instances (proof-app and test-app pattern) — intentional, the first proves the observer is wired before the second proves the no-op; do not flag the two-app pattern as waste.
- `pub fn perfect_bump_dash_cancel` in `bump.rs` — wider than `pub(crate)` needed. Pre-existing pattern matching other bump system functions; acceptable until a cleanup pass targets visibility.
- `f32::from(u16::try_from(n).unwrap_or(u16::MAX))` pattern for safe u32→f32 conversion — appears 12+ times across spawn_cells_from_layout.rs, generate_node_sequence.rs, update_loading_bar.rs. Accepted as idiomatic in this codebase. Could be extracted to a shared helper in a future cleanup pass but is not a priority.
- `Node { ... }` in `fx/transition.rs` spawn calls — this is Bevy's UI layout component (renamed from `Style` in Bevy 0.18), not the game vocabulary term "Node" (a level). Not a vocabulary violation; Bevy owns this type name. Do not flag.
- `track_node_cleared_stats.rs` uses 5 named items from `crate::run::resources` (HighlightKind, HighlightTracker, RunHighlight, RunState, RunStats) — still below the 4+ glob trigger after the wave-3 refactor changed to an explicit import list. The explicit list is now correct; file no longer uses the wildcard import. Do not flag as of feature/wave-3-offerings-transitions.
- `detect_combo_and_pinball.rs` `ComboMessages` SystemParam — `pub(crate)` because it is a named `SystemParam` that must be reachable by Bevy internals. The struct itself is defined and consumed in the same file. Acceptable; Bevy requires SystemParam to be pub(crate) at minimum.
- `complete_transition_out.rs` doc comment "Will be replaced by timed transition animation in a later commit" — stale; animation is already in FxPlugin (wave 3 complete). Flag as stale comment.
- `MassDestruction` and `FirstEvolution` in `HighlightKind` — CLOSED as of feature/wave-3-offerings-transitions. Both now have dedicated detection systems (`detect_mass_destruction.rs`, `detect_first_evolution.rs`) with full test coverage.
- `CLUTCH_CLEAR_THRESHOLD`, `FAST_CLEAR_FRACTION`, `PERFECT_STREAK_THRESHOLD`, `MASS_DESTRUCTION_COUNT` — module-level constants in `run/resources.rs`. Defined as `pub` at crate root (not `pub(crate)`). Acceptable since they are consumed by multiple files across the run domain.
- `EvolutionRecipe` / `EvolutionIngredient` in `chips/definition.rs` — vocabulary correct; "evolution" is the established game term for chip combination (ChipInventory doc references evolutions). Do not flag.
- `ChipRegistry::insert` clones `name` twice (once for HashMap key, once for `order` vec) — this is the correct pattern given `insert` takes the `ChipDefinition` by value and must store both the key and preserve the definition. Do not flag.
- Scenario runner checker files (`breaker-scenario-runner/src/invariants/checkers/`) do NOT have `//!` module-level doc comments — this is the established pattern for all checkers in this directory (bolt_in_bounds.rs, valid_breaker_state.rs, etc.). Do not flag new checkers for missing module docs.
- `OfferingConfig::seen_decay_factor` field — defined in the struct but not used inside `offering.rs` itself; the field exists so callers can pass a complete config bundle (the actual decay recording happens in the caller, `generate_chip_offerings`). Intentional data-bag design. Do not flag as dead code.
- Checker files in `breaker-scenario-runner/src/invariants/checkers/` that lack `//!` module docs — pre-existing pattern in this directory; all existing checkers (e.g., bolt_in_bounds.rs, valid_breaker_state.rs) also omit module docs. Only flag if the project convention is updated.

## Phase 5c / Phase 6 (feature/wave-3-offerings-transitions, 2026-03-23)
- `spawn_bolt.rs` calls `breaker_query.iter().next()` twice to get y and x separately — minor redundancy, but pre-existing; do not flag as new issue introduced by this PR.
- `Wall` and `Cell` markers are `pub(crate)` — intentional; only spawned internally. Do not flag `pub(crate)` visibility on these markers.
- `PreviousScale` struct mirrors `Scale2D` field layout (x: f32, y: f32) rather than being a newtype over Vec2 — intentional; matches `Scale2D`'s non-newtype design for easy field-by-field lerp in `propagate_scale`.
- `save_previous_positions` stale name — CLOSED as of Wave 1 spatial2d review (2026-03-23). Function has been renamed to `save_previous` in the current code.
- The `#[require]` tests for `Bolt`, `Breaker`, `Cell`, and `Wall` all verify negative cases (cleanup components NOT auto-inserted) — intentional regression guard pattern. Do not flag as over-testing.

## Wave 1 + spatial2d VFX (feature/wave-3-offerings-transitions, 2026-03-23, simplify pass)
- `tick` helper is module-local in every test module (74 files). This is the established pattern for the whole codebase — not a duplication issue. Do not flag per-module `tick` helpers as reuse candidates.
- `animate_shockwave` in shockwave.rs tests spawns `Assets<ColorMaterial>` directly via `init_resource` — correct; the game's full asset pipeline is not needed in unit tests. The HDR `ColorMaterial` literal `Color::linear_rgba(0.0, 4.0, 4.0, 0.9)` with `AlphaMode2d::Blend` is repeated across 6 VFX tests; this is intentional (each test is self-contained) rather than a shared constant, consistent with test-isolation norms here.
- `assert_standard_shockwave_components` helper in shockwave.rs tests — correctly extracted from the first test that spawned with all standard components; reduces duplication within the file only. Not a cross-file utility; do not flag.
- `ShieldBehavior` field rename: `orbit_count/radius/speed/hp/color_rgb` → `count/radius/speed/hp/color_rgb` — vocabulary simplification approved in simplify pass. RON files must be updated to match; this is a RON-breaking rename.

## Wave E — highlight scoring + popups (feature/spatial-physics-extraction, 2026-03-24)
- `config_f32(val: u32) -> f32` private helper in `select_highlights.rs` — module-private 2-line helper for lossless u32→f32 via u16::try_from. Consistent with the established `f32::from(u16::try_from(n).unwrap_or(u16::MAX))` pattern throughout the codebase. Do not flag.
- `_ => unreachable!()` arm in `score_highlight` after the `match highlight.kind` block — the binary-type guard above the match uses an exhaustive early-return, so this arm is structurally unreachable. Intentional invariant enforcer. Do not flag.
- `_config: Res<HighlightConfig>` parameter in `detect_first_evolution` — the cap check that required this param has been removed (cap moved to run-end selection). The `_` prefix suppresses the unused warning but leaves a dead Bevy system param. This is a known cleanup item: the parameter should be removed entirely. Flag if still present in a future cleanup pass.
- 11 `max_expected_*` fields on `HighlightDefaults` (one per scored HighlightKind) — parallel constants for normalization ceilings. This is parameter sprawl by design (all configurable via RON). The alternative (HashMap<HighlightKind, f32>) would require a custom RON deserializer. Acceptable given the RON-first constraint. Do not flag the count of fields.
- `select_highlights` returns `Vec<usize>` (indices into the input slice) rather than returning `RunHighlight` values directly — intentional to let callers control materialization and avoid allocation. Do not flag.

## Wave 1 — spatial2d new systems (feature/wave-3-offerings-transitions, 2026-03-23)
- `compute_globals` uses a two-pass loop with a `HashMap<Entity, (Vec2, Rot2, (f32, f32))>` parent cache — intentional pattern to avoid conflicting mutable borrows. Do not flag the HashMap allocation as unnecessary.
- `propagate_position`, `propagate_rotation`, `propagate_scale` still registered in plugin alongside the new `compute_globals` + `derive_transform` — these are NOT dead code; they still write `Transform` components via Bevy's parent hierarchy mechanism. `derive_transform` writes from Global* components and is the *new* path; `propagate_*` remain as the old complementary path. The coexistence is intentional during the migration. Do not flag these as duplicates unless a future wave removes them.
- `save_previous` (save_previous.rs) splits into four separate sub-queries (query_pos, query_rot, query_scale, query_vel) — intentional; Bevy requires separate queries for separate borrows of the same component family when the filter (`With<InterpolateTransform2D>`) differs. Do not flag as query duplication.
- Scale interpolation in `derive_transform` uses manual lerp `prev.x + (g_scale.x - prev.x) * alpha` rather than a `lerp` call — intentional; `f32` has no built-in `lerp` in stable Rust at the time of writing; this is the idiomatic manual form. Do not flag.
- `GlobalScale2D` fields `x: f32, y: f32` match `Scale2D`'s non-newtype layout — intentional symmetry. Do not flag as inconsistent with `GlobalPosition2D(Vec2)` newtype style; scale requires field-level access for the propagation math.
- `derive_transform` has the interpolation guard `if interp.is_some()` repeated three times (pos, rot, scale) — intentional; each field can be independently interpolated or not (the guard is per-field, not per-entity). Do not flag as duplication.

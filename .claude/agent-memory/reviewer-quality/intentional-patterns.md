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
- Scenario runner checker files (`breaker-scenario-runner/src/invariants/checkers/`) do NOT have `//!` module-level doc comments — this is the established pattern for all checkers in this directory (bolt_in_bounds.rs, valid_breaker_state.rs, etc.). Do not flag new checkers for missing module docs.
- `OfferingConfig::seen_decay_factor` field — defined in the struct but not used inside `offering.rs` itself; the field exists so callers can pass a complete config bundle (the actual decay recording happens in the caller, `generate_chip_offerings`). Intentional data-bag design. Do not flag as dead code.
- Checker files in `breaker-scenario-runner/src/invariants/checkers/` that lack `//!` module docs — pre-existing pattern in this directory; all existing checkers (e.g., bolt_in_bounds.rs, valid_breaker_state.rs) also omit module docs. Only flag if the project convention is updated.

# quality-reviewer Memory

## Intentional Patterns (Do Not Flag)

- `existing.iter().next().is_some()` — used as a guard in `spawn_lives_display`. `Query::is_empty()` does exist in Bevy 0.18.1 (used in `spawn_side_panels`), so `.iter().next().is_some()` is a minor inconsistency but not wrong. Flag if seen in new code; prefer `.is_empty()` going forward.
- `spawn_side_panels` uses `!existing.is_empty()` as its singleton guard — this is the preferred form in this codebase.
- `let _ = &playfield;` in `spawn_breaker` was a future-centering placeholder — now removed; `reset_breaker` uses `f32::midpoint(playfield.left(), playfield.right())` for robust centering. Do not flag the midpoint usage.
- `let _ = &defaults;` in `apply_archetype_config_overrides` — intentional placeholder to keep unused parameters in the signature without compiler warnings, noted for hot-reload. Do not flag.
- `allow(clippy::missing_const_for_fn)` on simple tuple-struct getters like `BoltVelocity::new` — Bevy structs often can't be `const fn` due to trait bounds; suppress is legitimate.
- `allow(clippy::cast_precision_loss)` on `col_idx as f32` / `row_idx as f32` — grid indices won't exceed f32 precision limits; suppress is reasonable.
- `allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)` for timer display ceiling — `timer.remaining.ceil().max(0.0) as u32` is always safe given the `max(0.0)` guard.
- Heavy use of `.unwrap()` in test code only — all production paths use `if let`, `let Ok(...) = ... else { return; }`, or `let Some(...) = ... else { return; }`. No unwraps in production code paths. This is intentional and correct.
- `#[cfg(all(test, not(target_os = "macos")))]` on integration tests in `game.rs` and `app.rs` — platform guard for headless test stability. Do not flag.
- `#[allow(dead_code)]` on `BumpPerformed` and `CellDestroyed` struct definitions — these are message types; the fields are used by systems via pattern matching, but the allow is needed because the derive macro doesn't see all usage sites. Intentional.
- Double-insert in `init_breaker_params` (two separate `.insert((...))` calls on the same entity) — Bevy has a 15-component tuple limit; splitting is the correct workaround. Do not flag.

## Vocabulary Decisions
- `format_lives` in `life_lost.rs` is a private helper that formats a `u32` into a display string. The term "lives" is correct game vocabulary here (it's a count of `LivesCount`). Not a vocabulary violation.
- `fire_consequences` in `bridges.rs` is a private helper — "consequence" is used in its precise game-system sense (from `Consequence` enum). Not a violation.

## Test Coverage Standards by Domain

- Bump system (`breaker/systems/bump.rs`): Very high coverage. Grade functions (pure), `update_bump` (integration), `grade_bump` (integration), combined pipeline, BoltServing guard, FixedUpdate input-loss regression, and `perfect_bump_dash_cancel` all tested.
- CCD physics (`physics/ccd.rs`, `bolt_cell_collision.rs`, `bolt_breaker_collision.rs`): Comprehensive. All collision surfaces, multi-bolt, cascade prevention (2-frame), MAX_BOUNCES cap, wall vs cell distinction, overlap resolution all covered.
- Breaker state machine (`breaker/systems/dash.rs`): All 4 state transitions tested, easing correctness, frame-rate independence, timer initialization. Very thorough.
- Node completion (`run/node/systems/track_node_completion.rs`): All branches covered (required, non-required, zero, remaining).
- Run end paths (`run/systems/handle_node_cleared.rs`, `handle_timer_expired.rs`): All three outcomes (node-transition, win, no-op) tested.

## Documentation Conventions

- Module-level `//!` doc comments are used consistently on all `.rs` files.
- Public types, functions, and helpers all have `///` doc comments with field-level documentation.
- Private helpers within systems often lack doc comments (accepted pattern when the function name is self-describing and a module doc covers intent).
- Units are documented inline on component fields (e.g., "world units per second", "radians", "seconds") — this is established convention.
- `#[must_use]` is applied to pure query methods and value-returning helpers. Follow this pattern for new `impl` blocks.

## Vocabulary Decisions (continued)
- `upgrade` module name, `UpgradeKind`, `UpgradeDefinition`, `UpgradeRegistry`, `UpgradeOffers`, `UpgradeSelected`, `UpgradeCard` — infrastructure wrappers around Amp/Augment/Overclock concepts; these names are acceptable. Only player-facing UI strings must avoid "power-up", "upgrade", etc.

## Recent Reviews

- [review-fix-review-findings-2026-03-16.md](review-fix-review-findings-2026-03-16.md) — review of fix/review-findings branch: init_breaker_params, animate_fade_out migration to UI, spawn_breaker centering, handle_cell_hit Vec dedup, spawn_cells_from_layout position tests
- [review-screen-domain-2026-03-16.md](review-screen-domain-2026-03-16.md) — review of run_setup, pause_menu, upgrade_select, loading/seed_upgrade_select_config, and screen plugin
- [review-upgrade-infrastructure-2026-03-16.md](review-upgrade-infrastructure-2026-03-16.md) — review of feature/upgrade-infrastructure: definition/registry/seed_upgrade_registry/spawn_upgrade_select/handle_upgrade_input
- [review-feature-review-fixes-2-2026-03-16.md](review-feature-review-fixes-2-2026-03-16.md) — review of feature/review-fixes-2: chip_select screen, chips domain, seed_chip_registry/select_config, vocabulary rename complete

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

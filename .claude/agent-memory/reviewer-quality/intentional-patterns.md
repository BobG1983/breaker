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
- `#[cfg(all(test, not(target_os = "macos")))]` on integration tests — platform guard.
- `#[allow(dead_code)]` on BumpPerformed and CellDestroyed — message type derive macro limitation. Intentional.
- Double-insert in `init_breaker_params` — Bevy 15-component tuple limit workaround.
- `handle_cell_hit.rs` `peekable().peek().is_none()` early-return — idiom smell, flag if still present in new code.
- `scenario_actions.len() as u32` in lifecycle.rs — safe in practice.

## Vocabulary Decisions
- `format_lives` in `life_lost.rs` — "lives" is correct game vocabulary (count of `LivesCount`).
- `fire_consequences` in `bridges.rs` — "consequence" used in its precise game-system sense.
- `upgrade` module/type names — infrastructure wrappers around Amp/Augment/Overclock; acceptable.
- `ChaosDriver` — renamed from `ChaosMonkey` in feature/scenario-coverage-expansion. Rename is complete in production code (`src/input.rs`). Test bodies still use `monkey` as local variable names (`let mut monkey = ChaosDriver::new(...)`) — acceptable in test-only code.
- `HybridInput` scripted phase boundary: doc says `0..scripted_frames` exclusive, implementation uses `frame < scripted_frames` (correct). The edge-case test probes frame 99 (not frame 100); comment in test says "last scripted frame". This is correct — 99 is inside scripted phase when scripted_frames=100.

---
name: Validation History
description: Historical build validation results — test counts, formatting changes, and build failures over time
type: reference
---

# Validation History

## 2026-03-16, fix/review-findings (latest)
- Format: PASS (1 file auto-fixed: animate_fade_out.rs)
- Clippy: FAIL (3 errors, 4 warnings — spawn_cells_from_layout.rs and animate_fade_out.rs)
- Tests: FAIL (362 passed, 5 failed, 0 ignored)
- Failures: 4 in animate_fade_out tests + 1 in dash (frame-rate independence)
- Root cause: animate_fade_out tests not advancing timer/updating alpha in loop
- Status: Blocking issues in ui/systems/animate_fade_out.rs test logic and spawn_cells_from_layout.rs lint violations

## 2026-03-16, main (latest)
- Format: PASS (no files needed formatting)
- Clippy: PASS (no warnings or errors)
- Tests: PASS (355 passed, 0 failed, 0 ignored)
- Change: +9 tests since previous validation (346 → 355) — aegis multiplier tests added
- Status: Main branch is clean and ready for development
- Modified files: assets/archetypes/aegis.archetype.ron, src/breaker/behaviors/consequences/bolt_speed_boost.rs, src/breaker/behaviors/init.rs

## 2026-03-16, main (earlier)
- Format: PASS (3 files auto-formatted: life_lost.rs, spawn_side_panels.rs, spawn_timer_hud.rs)
- Clippy: PASS (no warnings or errors)
- Tests: PASS (346 passed, 0 failed, 0 ignored)
- Change: +1 test since previous validation (345 → 346)
- Status: Main branch is clean and ready for development

## 2026-03-16, main (earlier)
- Format: PASS (2 files auto-formatted: handle_cell_hit.rs, track_node_completion.rs)
- Clippy: Build failed (compilation error prevents linting)
- Tests: Build failed (compilation error prevents testing)
- Error: src/run/systems/handle_timer_expired.rs:32 — NextState<GameState>.get() method does not exist in Bevy 0.18.1
- Root cause: NextState API in Bevy 0.18.1 has no public read method; only .set() is available
- Status: Blocking issue requires developer fix

## 2026-03-16, main (earlier)
- Format: PASS (1 file auto-formatted: app.rs)
- Clippy: PASS (no warnings or errors)
- Tests: PASS (341 passed, 0 failed, 0 ignored)
- Status: Main branch is clean and ready for development

## 2026-03-13, main
- Format: PASS (1 file auto-formatted: breaker/behaviors/active.rs)
- Clippy: PASS (no warnings or errors)
- Tests: PASS (339 passed, 0 failed, 0 ignored)
- Change: +95 tests since last main validation (2026-03-13) — significant test suite expansion
- Status: Main branch is clean and ready for development

## 2026-03-13, refactor/extract-wall-domain
- Format: PASS (no files needed formatting)
- Clippy: PASS (no warnings or errors)
- Tests: PASS (244 passed, 0 failed, 0 ignored)
- Status: Wall domain extraction is complete and builds cleanly

## 2026-03-13, main (earlier)
- Format: PASS (1 file auto-formatted: run/plugin.rs)
- Clippy: PASS (no warnings or errors)
- Tests: PASS (244 passed, 0 failed, 0 ignored)
- Change: +23 tests since 2026-03-12 (added node clearing and cell handling tests)
- Status: Main branch is clean and ready for development

## 2026-03-12, main
- Format: PASS (2 files auto-formatted: bolt/components.rs, breaker/systems/dash.rs)
- Clippy: PASS (no warnings or errors)
- Tests: PASS (221 passed, 0 failed, 0 ignored)
- Change: +1 test since previous validation
- Status: Main branch is clean and ready for development

## 2026-03-12, feature/grade-dependent-bump-cooldown
- Format: PASS (1 file auto-formatted: bolt_breaker_collision.rs)
- Clippy: 1 warning (missing_const_for_fn in cooldown_for_grade)
- Tests: 208 passed, 0 failed, 0 ignored
- Change: +8 tests (bump grade cooldown mechanics)
- Note: cooldown_for_grade in breaker/systems/bump.rs could be const; flagged by clippy nursery lint

## 2026-03-12, feature/bump-timing-rework
- Format: PASS (1 file auto-formatted: bump_visual.rs)
- Clippy: PASS (no warnings or errors)
- Tests: 200 passed, 0 failed, 0 ignored
- Change: multi-line spawn chain condensed to single line per rustfmt

## 2026-03-12, main (earlier)
- Format: PASS (1 file auto-formatted: tilt_visual.rs)
- Clippy: PASS (no warnings or errors)
- Tests: 184 passed, 0 failed, 0 ignored
- Change: refactored tilt_visual tests to use parametrized helper function

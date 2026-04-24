# File Splits — port branch sweep (2026-04-23)

## Context

Port branch (`feature/port-breaker-to-rantzsoft-dmg`) is mid-flight on TODO #1.
The 2026-04-22 sweep (TODO #0 wrap) split 8 files in `rantzsoft_dmg/`; since then
several of those files have re-grown past threshold and a few siblings have
crossed 400 for the first time.

Scope probed this pass: every file surfaced by `git status` as modified on this
branch, every previous-sweep target (to detect re-growth), and the full
`rantzsoft_dmg/src/` + `rantzsoft_dmg/tests/` tree. Line counts derived from
Read-tool end-of-file probes — no bash was available in this session.

**Caveat**: I did not exhaustively scan every `.rs` under `breaker-game/src/` — the
workspace has hundreds of files and without bash I can only probe individually.
Every modified-by-this-branch file in `breaker-game/src/` was checked; all were
well under 400. A bash-enabled pass post-merge should confirm no new offenders.

## File Length Review

### Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `breaker-game/src/bolt/systems/bolt_lost/tests/lost_detection_tests.rs` | 721 | n/a | 721 | ~20 | A: test-only → sub-split by concern | HIGH |
| `rantzsoft_dmg/src/systems/apply_damage.rs` | 573 | 49 | 523 | ~19 | A: test extraction | HIGH |
| `rantzsoft_dmg/src/systems/apply_damage_boosts/tests.rs` | 566 | n/a | 566 | ~17 | C (monitor) — under 800 | LOW |
| `rantzsoft_dmg/src/systems/apply_heal/tests.rs` | 512 | n/a | 512 | ~23 | C (monitor) — under 800 | LOW |
| `rantzsoft_dmg/src/cells/behaviors/sequence/tests/group_c.rs` | 513 | n/a | 513 | n/a | C (monitor) — under 800 | LOW |
| `breaker-game/src/bolt/systems/bolt_lost/tests/extra_bolt_tests.rs` | 496 | n/a | 496 | n/a | C (monitor) — under 800 | LOW |
| `rantzsoft_dmg/src/app_ext/tests.rs` | 521 | n/a | 521 | ~17 | C (monitor) — under 800 | LOW |
| `rantzsoft_dmg/src/plugin/tests.rs` | 718 | n/a | 718 | ~20 | C (monitor) — 82 lines from threshold | MEDIUM |
| `rantzsoft_dmg/src/systems/handle_kill/tests.rs` | 650 | n/a | 650 | ~23 | C (monitor) — under 800 | LOW |
| `rantzsoft_dmg/tests/pipeline_integration/damage.rs` | 436 | n/a | 436 | ~9 | C (monitor) — under 800 | LOW |
| `rantzsoft_dmg/src/lib.rs` | 408 | ~130 | ~278 | 8 | A' (lib-root test extraction) | MEDIUM |
| `rantzsoft_dmg/src/components/damage_boost_stack.rs` | 398 | n/a | n/a | n/a | under threshold — monitor | — |
| `breaker-game/src/bolt/systems/bolt_cell_collision/tests/damage_messages.rs` | 432 | n/a | n/a | n/a | A (test concern split) | MEDIUM |

### Priority Guide
- **HIGH**: 500+ lines requiring immediate split; either 700+ test-only (context bomb) or system+tests combined over 550
- **MEDIUM**: 401–500, or 700+ test-only files within one split of the 800 sub-split threshold
- **LOW**: Over threshold but small enough that a simple extraction or monitor pass is fine

### mod.rs Violations
None observed in the directories probed.

---

## Refactor Spec Hints

### 1. `breaker-game/src/bolt/systems/bolt_lost/tests/lost_detection_tests.rs` — HIGH

- Source file: `breaker-game/src/bolt/systems/bolt_lost/tests/lost_detection_tests.rs`
- Total lines: 721 (all tests — this file lives inside an already-split `tests/` directory)
- Strategy: **C — already-extracted oversized tests file; sub-split by concern**
- Target structure (keep `bolt_lost/tests/` directory; replace single oversized file):
  ```
  breaker-game/src/bolt/systems/bolt_lost/tests/
    mod.rs                             // add: mod lost_detection;
    lost_detection/
      mod.rs                           // mod below_floor; mod above_ceiling;
                                       // mod side_exits; mod threshold_edges;
      below_floor.rs                   // tests where bolt falls below the playfield
      above_ceiling.rs                 // tests where bolt escapes above
      side_exits.rs                    // left/right boundary tests
      threshold_edges.rs               // exact-boundary/off-by-one edge cases
  ```
- Test groups (identify by test-name scan when executing — likely groupings):
  - `below_floor.rs`: all tests whose name contains `below`, `floor`, `y_negative`, or `bottom`
  - `above_ceiling.rs`: tests containing `above`, `ceiling`, `top`, `y_positive`
  - `side_exits.rs`: tests with `left`, `right`, `x_` boundary names
  - `threshold_edges.rs`: `exact_`, `boundary_`, `edge_`, `off_by_one` named tests
- Imports per sub-file: copy existing `use` block from `lost_detection_tests.rs`; adjust `use super::*;` → `use super::super::*;` (two directories up to the `bolt_lost/` tests root). Shared helpers stay in the existing `tests/helpers.rs`.
- Parent change: **`tests/mod.rs` edit** — replace `mod lost_detection_tests;` with `mod lost_detection;` (or keep the old module name, entirely orchestrator preference).
- Delegate: writer-code can execute this refactor directly

### 2. `rantzsoft_dmg/src/systems/apply_damage.rs` — HIGH

- Source file: `rantzsoft_dmg/src/systems/apply_damage.rs`
- Total lines: 573 (prod: 49, tests: 523 starting at line 51)
- Strategy: **A — test extraction** (matches 2026-04-22 pattern for siblings)
- Doc comment to preserve: file-level `//!` block (lines 1–6) moves to `system.rs`.
- Target structure:
  ```
  rantzsoft_dmg/src/systems/
    apply_damage/
      mod.rs      // pub(crate) use system::apply_damage; mod system; #[cfg(test)] mod tests;
      system.rs   // file-level doc + production (lines 1–49)
      tests.rs    // all tests (lines 51–573)
  ```
- Tests file size: 523 lines — below 800 threshold, keep as single `tests.rs`.
- Test groups (kept together — already logically cohesive):
  - Basic damage application / HP decrement
  - Killing-blow `KilledBy` insertion (`msg.attributed_to.or(msg.dealer)`)
  - First-kill-wins on same-tick multi-damage
  - Dead / missing-marker / missing-Hp skips
  - B153 contract tests (no `Without<Invulnerable>` on query)
- Imports needed in `system.rs`: current imports from lines 8–14
- Imports needed in `tests.rs`: `use super::system::*;` plus existing test-block imports (line 53 onwards)
- Re-exports in `mod.rs`: `pub(crate) use system::apply_damage;` — matches `systems/mod.rs` line 12
- Parent change: **NONE** — `src/systems/mod.rs` line 3 (`mod apply_damage;`) resolves transparently
- Delegate: writer-code can execute this refactor directly

### 3. `rantzsoft_dmg/src/lib.rs` — MEDIUM

- Source file: `rantzsoft_dmg/src/lib.rs`
- Total lines: 408 (prod: ~130 through line 128; tests: lines 130–407; `#[cfg(test)] mod tests` starts at 130)
- Strategy: **A' — lib-root test extraction** (cannot convert `lib.rs` to a directory; instead extract tests to a sibling file)
- Doc comment to preserve: stays in `lib.rs` (the crate-level `//!` block lines 1–96).
- Target structure:
  ```
  rantzsoft_dmg/src/
    lib.rs                   // UNCHANGED except the #[cfg(test)] mod tests { ... } block is replaced with:
                             //     #[cfg(test)] mod crate_root_tests;
    crate_root_tests.rs      // entire inline tests module body (currently lines 130–407)
  ```
- Imports in `crate_root_tests.rs`: move the `use crate::*;` / `use crate::...;` imports inside each test into the file's top-level `use` block, or keep them scoped as-is. Each test already has its own `use`s — safe to keep that shape.
- Re-exports: none needed — `crate_root_tests.rs` is a private sibling test module.
- Why not a `tests/` subdirectory: `lib.rs` is the crate root. A `#[path = "..."]` attribute could let tests live in a directory but the sibling-file form is simpler and matches the standard Rust 2018 form for lib tests.
- Delegate: writer-code can execute this refactor directly

### 4. `rantzsoft_dmg/src/plugin/tests.rs` — MEDIUM (monitor)

- Source file: `rantzsoft_dmg/src/plugin/tests.rs`
- Total lines: 718 (test-only — already extracted from `plugin.rs` on 04-22)
- Strategy: **C — monitor**; currently 82 lines below the 800-line sub-split trigger
- Expected behavior: the `RantzDmgPlugin` registers on many `App` shapes and TODO #1 is landing new test coverage for per-domain wiring. If it crosses 800, sub-split by:
  - `derive_contracts.rs` (B40–B44 type-contract tests)
  - `despawn_registration.rs` (B45)
  - `per_t_isolation.rs` (B46)
  - `chain_ordering.rs` (B47–B49)
  - `fixed_post_update.rs` (B50)
  - `headless_noninjection.rs` (B52, B53)
- No action needed this pass; re-flag after TODO #1 ships.

### 5. `breaker-game/src/bolt/systems/bolt_cell_collision/tests/damage_messages.rs` — MEDIUM

- Source file: `breaker-game/src/bolt/systems/bolt_cell_collision/tests/damage_messages.rs`
- Total lines: 432 (test-only — already inside an extracted `tests/` directory)
- Strategy: **C — sub-split by damage concern** (but still under 800, so MEDIUM rather than HIGH)
- If sub-splitting, suggested groups (determine from test names during execution):
  - `dealer_attribution.rs` (who owns the damage)
  - `amount_resolution.rs` (boost / vulnerable stack interaction)
  - `per_t_marker.rs` (DamageDealt<Cell> marker behavior after TODO #1 port)
- Alternative: defer until TODO #2 (which consolidates `hazard/` + `protocol/` → `mutators/`) since the cell damage pipeline moves in that todo — avoid churning tests in flight.
- Recommendation: **defer split until TODO #1 merges**. Re-evaluate in next sweep.

### 6. `rantzsoft_dmg/src/systems/apply_damage_boosts/tests.rs` — LOW (monitor)

- Total: 566 (was 481 on 04-22; +85 lines in three days).
- Under 800 sub-split threshold but trending up. If TODO #1 adds more per-`T`-marker boost tests, this will cross.
- Next-sweep trigger: 700+ lines → sub-split by lane (`persistent`, `one_shot`, `both_lanes`, `multi_entry`, `pass_through`).

### 7. `rantzsoft_dmg/src/systems/apply_heal/tests.rs` — LOW (monitor)

- Total: 512 (close to the 04-22 count of 513 — stable). No action.

### 8. `rantzsoft_dmg/src/app_ext/tests.rs` — LOW (monitor)

- Total: 521 (was 514). Stable. No action.

### 9. `rantzsoft_dmg/src/systems/handle_kill/tests.rs` — LOW (monitor)

- Total: 650 (was 652 on 04-22). Stable. No action.

### 10. `rantzsoft_dmg/tests/pipeline_integration/damage.rs` — LOW (monitor)

- Total: 436 (was 422). Up 14 lines. Stable. No action; already inside a `#[path]`-based test directory.

### 11. `breaker-game/src/bolt/systems/bolt_lost/tests/extra_bolt_tests.rs` — LOW (monitor)

- Total: 496. Extracted-tests file; under 800. No action, but within 4 lines of crossing into MEDIUM territory next sweep.

### 12. `breaker-game/src/cells/behaviors/sequence/tests/group_c.rs` — LOW (monitor)

- Total: 513 (modified on this branch per git status). Already inside a sub-split tests directory. Under 800; no action.

### Borderline (under 400, monitor next sweep)

| File | Total | Notes |
|------|-------|-------|
| `rantzsoft_dmg/src/components/damage_boost_stack.rs` | 398 | 2 lines under; effectively at threshold |
| `rantzsoft_dmg/src/systems/bolt_cell_collision/system.rs` | 389 | stable; watch if per-T porting adds more branches |
| `rantzsoft_dmg/src/sets.rs` | 373 | up from 322 on 04-22; trending up |
| `rantzsoft_dmg/src/systems/detect_deaths.rs` | 337 | stable |
| `rantzsoft_dmg/src/messages/damage_dealt.rs` | 317 | test-heavy; TODO #1 will likely push this over |
| `rantzsoft_dmg/src/systems/handle_kill/tests.rs` helpers | — | — |

---

## Batching for Parallel Writer-Code

All HIGH/MEDIUM actionable splits touch disjoint files and are safe to parallel-launch.

- **Wave A (HIGH)**: `apply_damage.rs`, `lost_detection_tests.rs` — 2 writer-code agents
  (one in `rantzsoft_dmg/src/systems/`, one in `breaker-game/src/bolt/systems/bolt_lost/tests/`)
- **Wave B (MEDIUM)**: `lib.rs` test extraction — 1 agent, sibling-file form; DO NOT touch
  the doc-block or `#![cfg_attr]` lines.

The `damage_messages.rs` MEDIUM is deferred per §5 above; revisit post-TODO-#1.

## Post-Split Verification

After each split:
1. Orchestrator removes orphaned `.rs` files co-existing with new `foo/mod.rs` directory modules.
2. Run Basic Verification Tier (`cargo all-dtest` + `cargo all-dclippy` — see `.claude/rules/cargo.md`).
3. For `lib.rs` case specifically: re-run all doctests (`cargo dtest --doc`) since the compile_fail doctests live at file top and any accidental edit to them would mask a Behavior 13 regression.

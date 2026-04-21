# File Splits: protocol-burnout Full Verification Tier scan

Scan date: 2026-04-20
Branch: `feature/protocol-burnout` (pre-merge to develop)
Scope:
1. Verify the Burnout protocol module (new in this branch) is within limits.
2. Workspace-wide scan for any `.rs` file that has grown past 400 lines and is NOT already tracked by an open split todo.

## Burnout module result (in-branch)

Burnout was authored as a directory module with tests already sub-split by behavior group (A–F). Per-file sizes are well within limits:

| File | Lines | Notes |
|------|-------|-------|
| `protocol/protocols/burnout/mod.rs` | 11 | wiring only — clean |
| `protocol/protocols/burnout/system.rs` | 395 | single-concern prod + 5 systems + activate/register — under threshold |
| `protocol/protocols/burnout/tests/helpers.rs` | 298 | |
| `protocol/protocols/burnout/tests/activate.rs` | 185 | |
| `protocol/protocols/burnout/tests/components.rs` | 167 | |
| `protocol/protocols/burnout/tests/cleanup_node.rs` | 169 | |
| `protocol/protocols/burnout/tests/on_bump.rs` | 298 | |
| `protocol/protocols/burnout/tests/amplify.rs` | 326 | |
| `protocol/protocols/burnout/tests/register.rs` | 436 | **LOW** — over 400, awareness only |
| `protocol/protocols/burnout/tests/update_heat.rs` | 455 | **LOW** — over 400, awareness only |
| `protocol/protocols/burnout/tests/tick_speed_boost.rs` | 149 | |
| `protocol/protocols/burnout/tests/sentinel.rs` | 44 | |
| `protocol/protocols/burnout/tests/ron_asset.rs` | 59 | |
| `protocol/protocols/burnout/tests/plugin.rs` | 23 | |
| `protocol/protocols/burnout/tests/mod.rs` | 12 | wiring only |

**Burnout assessment**: no action required for this merge. `update_heat.rs` (455 lines, 16 behaviors C1–C14) and `register.rs` (436 lines) are both already per-group test files; they sit just over the 400 flag-for-awareness band. Only split if they grow past ~700 lines by adding further behaviors.

## Workspace-wide scan — not-tracked HIGH items

A workspace-wide scan turned up 6 files ≥1000 lines. Two are already covered by the open todo `detail/2026-04-17-file-splits.md` (`apply_heal.rs`, `death_pipeline/plugin.rs`). **The other four are HIGH priority and are NOT currently tracked by any open todo — they predate this branch but must be split before they grow further.**

### File Length Review

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `hazard/hazards/volatility.rs` | 1984 | 186 | 1797 | 71 | A: test extraction + C sub-split into Groups A–F | HIGH |
| `hazard/hazards/echo_cells.rs` | 1851 | 224 | 1626 | 74 | A: test extraction + C sub-split by behavior-number bands | HIGH |
| `hazard/hazards/fracture.rs` | 1573 | 172 | 1400 | 69 | A: test extraction + C sub-split by behavior-number bands | HIGH |
| `hazard/hazards/cascade/tests.rs` | 1382 | 0 | 1382 | 46 | C: already-extracted test file, sub-split into Groups A–G+ | HIGH |
| `shared/death_pipeline/systems/tests/apply_heal.rs` | 1897 | 0 | 1897 | 60 | C (covered by open todo `2026-04-17-file-splits.md`) | (tracked) |
| `shared/death_pipeline/plugin.rs` | 1034 | 136 | 893 | 17 | A (covered by open todo `2026-04-17-file-splits.md`) | (tracked) |

All paths are relative to `breaker-game/src/`.

### Priority Guide
- **HIGH**: 1000+ lines, or 800+ test lines
- **MEDIUM**: 501-999 lines
- **LOW**: 400-500 lines

---

## Refactor Specs (HIGH, not-tracked)

### 1. `hazard/hazards/volatility.rs` (1984 lines)

**Refactor spec hint:**
- Source file: `breaker-game/src/hazard/hazards/volatility.rs`
- Total lines: 1984 (prod: 186, tests: 1797 — `#[cfg(test)]` starts at line 187)
- Strategy: A (promote to `volatility/` directory) + C (sub-split tests into Groups A–F)
- Parent module: `hazard/hazards/mod.rs` declares `mod volatility;` — unchanged
- Doc comment to preserve on new `volatility/mod.rs`:
  ```
  //! Volatility hazard — interval-based cell recovery.
  //!
  //! Cells that are left untouched regrow HP in discrete interval ticks. Each
  //! interval emits a `HealDealt<Cell>` (capped via `HealCap::Max` against a
  //! lifted `Hp.max = 2× starting`) so growth rides the unified heal pipeline.
  //! Any `DamageDealt<Cell>` resets the cell's timer to zero — a cell "touched"
  //! within the interval stays at its current HP. Effective interval shrinks
  //! with hazard stacks (floored at 1.0s). Authoritative design doc:
  //! `docs/todos/detail/mod-system-design/hazards/volatility.md`.
  ```
- Target structure:
  ```
  hazard/hazards/volatility/
    mod.rs      // pub(crate) mod system; #[cfg(test)] mod tests;
                // Plus module-level doc comment above.
                // Re-export public items from system (activate, register, VolatilityConfig,
                // VolatilityTimer, any other pub/pub(crate) items referenced externally).
    system.rs   // everything before line 187 (production code: imports, constants,
                // VolatilityTimer, VolatilityConfig, activate, register,
                // attach_volatility_timers, reset_volatility_on_damage, volatility_grow_cells)
    tests/
      mod.rs              // mod helpers; mod group_a_interval; mod group_b_stacking;
                          //   mod group_c_damage_reset; mod group_d_attach;
                          //   mod group_e_multi_interval; mod group_f_cross_cutting;
                          //   mod activate_scaffold;
      helpers.rs          // lines ~198-365 — shared test helpers (test_app_playing,
                          //   test_app_playing_with_damage, spawn_cell_with_timer,
                          //   spawn_cell_with_timer_max, spawn_cell_no_timer, tick_with_dt,
                          //   heals_for, heal_collector_len, add_volatility_stacks,
                          //   default_config, install_default_config,
                          //   register_volatility_systems). All `pub(super)` visibility.
      group_a_interval.rs        // Group A — lines ~367-675 — interval/heal semantics
      group_b_stacking.rs        // Group B — lines ~676-887 — stacking and interval floor
      group_c_damage_reset.rs    // Group C — lines ~888-1215 — reset_volatility_on_damage
      group_d_attach.rs          // Group D — lines ~1216-1554 — attach_volatility_timers
      group_e_multi_interval.rs  // Group E — lines ~1555-1684 — pause/resume accumulation
      group_f_cross_cutting.rs   // Group F — lines ~1685-1964 — cross-cutting
      activate_scaffold.rs       // activate scaffold — lines ~1965-1983
  ```
- Imports needed in each tests sub-file (derive from the current `mod tests { use super::*; ... }` block — typically `use super::super::system::*;` plus whatever prelude imports the inner module pulls):
  ```rust
  use super::super::system::*;
  use super::helpers::*;
  use crate::prelude::*;
  // Plus Bevy Time, Duration, Entity, HealDealt imports as needed per file.
  ```
- Re-exports needed in `volatility/mod.rs`: mirror any `pub(crate)` item in the original file that is imported from `hazard/hazards/mod.rs` or from `hazard/plugin.rs`. Grep the current source for `pub(crate)` at the top level and re-export each.
- Delegate: writer-code can execute this refactor directly. The `// ══ Group X ══` comment bands in the source map 1:1 to the target files.

---

### 2. `hazard/hazards/echo_cells.rs` (1851 lines)

**Refactor spec hint:**
- Source file: `breaker-game/src/hazard/hazards/echo_cells.rs`
- Total lines: 1851 (prod: 224, tests: 1626 — `#[cfg(test)]` starts at line 225)
- Strategy: A (promote to `echo_cells/` directory) + C (sub-split tests by behavior-number bands)
- Parent module: `hazard/hazards/mod.rs` declares `mod echo_cells;` — unchanged
- Doc comment to preserve: read lines 1–10 of the source and copy verbatim to the new `mod.rs`.
- Target structure:
  ```
  hazard/hazards/echo_cells/
    mod.rs      // pub(crate) mod system; #[cfg(test)] mod tests;
                // Plus original module-level doc comment.
                // Re-export public items from system (activate, register, config/component types).
    system.rs   // everything before line 225
    tests/
      mod.rs      // mod helpers; mod formula; mod spawn_pending;
                  //   mod tick_expire; mod ghost_components; mod integration;
      helpers.rs         // shared test helpers (identify by grepping for `fn <name>(...) ->`
                         //   inside the test module before the first `#[test]` — lines ~225-520).
                         //   All `pub(super)` visibility.
      formula.rs         // Behaviors 1–6 (echo formula math) — ~lines 525-645
      spawn_pending.rs   // Behaviors 7–19 (track_destroyed_for_echoes / pending ghost lifecycle)
                         //   — ~lines 646-995
      ghost_components.rs // Behaviors 20–27b (spawned ghost component suite) — ~lines 996-1220
      tick_expire.rs     // Behaviors 28–30+ (timer tick + expiry) — ~lines 1221 onward
      integration.rs     // Remaining cross-cutting / register tests — tail of file
  ```
- The exact line bands above are approximate — writer-code should confirm by grepping `// Behavior N` markers in the source (the file numbers every test by behavior). Each sub-file takes a contiguous band.
- Imports needed: mirror the inner `mod tests { use super::*; use crate::prelude::*; ... }` header — same rewrite rules as volatility.
- Delegate: writer-code can execute. The `// Behavior N` markers are contiguous and non-overlapping; pick split boundaries at group-concept transitions (formula → spawn → components → tick → integration).

---

### 3. `hazard/hazards/fracture.rs` (1573 lines)

**Refactor spec hint:**
- Source file: `breaker-game/src/hazard/hazards/fracture.rs`
- Total lines: 1573 (prod: 172, tests: 1400 — `#[cfg(test)]` starts at line 173)
- Strategy: A (promote to `fracture/` directory) + C (sub-split tests by behavior-number bands)
- Parent module: `hazard/hazards/mod.rs` declares `mod fracture;` — unchanged
- Doc comment: read lines 1–10 and copy verbatim to new `mod.rs`.
- Target structure:
  ```
  hazard/hazards/fracture/
    mod.rs      // pub(crate) mod system; #[cfg(test)] mod tests;
    system.rs   // everything before line 173
    tests/
      mod.rs              // mod helpers; mod split_formula; mod spawn_debris;
                          //   mod debris_components; mod gating; mod integration;
                          //   mod activate;
      helpers.rs          // shared test helpers — `pub(super)` visibility
      split_formula.rs    // Behaviors 1–9 (num_splits math) — ~lines 255-435
      spawn_debris.rs     // Behaviors 10–13 (debris spawn count/positions) — ~lines 436-655
      debris_components.rs // Behaviors 14–16 (debris marker/collision suite/HP) — ~lines 656-830
      gating.rs           // Behaviors 17–18, 25–27 (cleanup + gate toggle) — gather from grep
      integration.rs      // Behaviors 19–24, 28+ (register + multi-death + recursion)
      activate.rs         // Behavior 28–29 (activate tuning) — tail of file
  ```
- Writer-code should verify band boundaries by grepping `// Behavior N` in the source — the file numbers every test.
- Delegate: writer-code can execute.

---

### 4. `hazard/hazards/cascade/tests.rs` (1382 lines)

**Refactor spec hint:**
- Source file: `breaker-game/src/hazard/hazards/cascade/tests.rs`
- Total lines: 1382 (prod: 0, tests: 1382 — already-extracted test file)
- Strategy: C (convert `tests.rs` to `tests/` directory, sub-split by existing `// Group X —` dividers)
- Parent module: `hazard/hazards/cascade/mod.rs` declares `#[cfg(test)] mod tests;` — UNCHANGED
- Doc comment: lines 1–6 of the source contain a `//!` — copy verbatim to new `tests/mod.rs`.
- Target structure:
  ```
  hazard/hazards/cascade/tests/
    mod.rs                 // mod helpers; mod group_a_formula; mod group_b_activate;
                           //   mod group_c_guards; mod group_d_single_death;
                           //   mod group_e; mod group_f; mod group_g;
                           // Plus original module-level `//!` doc comment.
    helpers.rs             // shared test helpers — lines ~27-112 — `pub(super)`
    group_a_formula.rs     // Group A — heal_per_neighbour formula — lines ~113-172
    group_b_activate.rs    // Group B — activate scaffold — lines ~173-244
    group_c_guards.rs      // Group C — No-op guards — lines ~245-424
    group_d_single.rs      // Group D — single-death/single-neighbour shape — lines ~425-703
    group_e.rs             // Group E — ~lines 704-887 (confirm by grepping `// Group E` marker)
    group_f.rs             // Group F — ~lines 888-1045
    group_g.rs             // Group G — ~lines 1046-1305 (full register integration per doc comment)
    group_h.rs             // Group H (if present) — ~lines 1306+ (tail)
  ```
- Imports: each sub-file needs `use super::super::*;` (since tests are under `cascade/tests/*`, `super::super::` reaches back into `cascade/`). Plus any `use` statements in the current `tests.rs` header.
- Re-exports needed in `tests/mod.rs`: none — test-only module.
- Writer-code should grep `// Group [A-Z] —` markers in the source to confirm exact boundaries; the file uses consistent `// ═══` band dividers.
- Delegate: writer-code can execute.

---

## Notes

- All four HIGH items follow the same pattern: single-file hazard module with inline `mod tests { ... }` (Strategy A test extraction) OR already-split `tests.rs` grown past 800 lines (Strategy C). Writer-code should apply the standard Strategy A / C procedure from `.claude/rules/file-splitting.md` — no novel decisions required.
- After each split: run `runner-linting` then `runner-tests` (Basic Verification Tier). No behavior changes, pure code movement.
- Batching hint: all four live under `breaker-game/src/hazard/hazards/` but each is a separate sibling module. They can be split in parallel without touching each other's files. `cascade/tests.rs` is the safest/cheapest (Strategy C only — no production-code move).

## Recommended batching for parallel writer-code agents

- **Batch 1** (parallel, all in `breaker-game/src/hazard/hazards/`): `volatility.rs`, `echo_cells.rs`, `fracture.rs`, `cascade/tests.rs`.
- No cross-file dependencies between these four — they are sibling hazard modules with isolated test scopes.
- Do NOT batch with the open `2026-04-17-file-splits.md` todo items (those touch `shared/death_pipeline/`, a different crate subdirectory, but serialize runner-tests regardless).

# File Splits: effect-system-refactor branch

Scan date: 2026-04-14
Branch: `feature/effect-system-refactor`
Scope: `breaker-game/src/effect_v3/` and related dispatch files

## File Length Review

### Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `effect_v3/conditions/evaluate_conditions.rs` | 3509 | 206 | 3303 | 64 | A: test extraction + sub-split | HIGH |
| `effect_v3/triggers/bump/bridges.rs` | 2718 | 311 | 2407 | 52 | A: test extraction + sub-split | HIGH |
| `effect_v3/walking/until.rs` | 2206 | 227 | 1979 | 27 | A: test extraction + sub-split | HIGH |
| `effect_v3/stacking/effect_stack.rs` | 1049 | 81 | 968 | 57 | A: test extraction + sub-split | HIGH |
| `effect_v3/effects/circuit_breaker/systems.rs` | 1031 | 64 | 967 | 25 | A: test extraction + sub-split | HIGH |
| `effect_v3/effects/piercing_beam/config.rs` | 1017 | 72 | 945 | 29 | A: test extraction + sub-split | HIGH |
| `effect_v3/effects/pulse/systems.rs` | 1015 | 137 | 878 | 25 | A: test extraction + sub-split | HIGH |
| `effect_v3/effects/entropy_engine/systems.rs` | 976 | 93 | 883 | 26 | A: test extraction + sub-split | HIGH |
| `effect_v3/effects/tether_beam/systems.rs` | 973 | 76 | 897 | 30 | A: test extraction + sub-split | HIGH |
| `effect_v3/triggers/death/bridges.rs` | 971 | 180 | 791 | 18 | A: test extraction | HIGH |
| `effect_v3/triggers/impact/bridges.rs` | 961 | 297 | 664 | 15 | A: test extraction | MEDIUM |
| `effect_v3/storage/spawn_stamp_registry/watchers/tests/bolt.rs` | 810 | 0 | 810 | 22 | C: already-extracted, sub-split | MEDIUM |
| `effect_v3/effects/pulse/config.rs` | 792 | 71 | 721 | 13 | A: test extraction | MEDIUM |
| `effect_v3/effects/shield/config.rs` | 773 | 87 | 686 | 16 | A: test extraction | MEDIUM |
| `effect_v3/walking/when.rs` | 733 | 49 | 684 | 14 | A: test extraction | MEDIUM |
| `effect_v3/walking/on.rs` | 713 | 72 | 641 | 16 | A: test extraction | MEDIUM |
| `effect_v3/effects/tether_beam/config.rs` | 683 | 132 | 551 | 18 | A: test extraction | MEDIUM |
| `effect_v3/effects/shockwave/systems.rs` | 629 | 80 | 549 | 18 | A: test extraction | MEDIUM |
| `effect_v3/walking/during.rs` | 624 | 83 | 541 | 8 | A: test extraction | MEDIUM |
| `effect_v3/effects/chain_lightning/config.rs` | 568 | 71 | 497 | 19 | A: test extraction | MEDIUM |
| `effect_v3/triggers/bolt_lost/bridges.rs` | 544 | 38 | 506 | 11 | A: test extraction | MEDIUM |
| `effect_v3/walking/once.rs` | 517 | 50 | 467 | 8 | A: test extraction | MEDIUM |
| `effect_v3/triggers/node/bridges.rs` | 509 | 77 | 432 | 9 | A: test extraction | MEDIUM |
| `effect_v3/effects/second_wind/config.rs` | 495 | 61 | 434 | 11 | A: test extraction | LOW |
| `effect_v3/triggers/time/bridges.rs` | 492 | 34 | 458 | 11 | A: test extraction | LOW |
| `effect_v3/dispatch/reverse_dispatch.rs` | 470 | 123 | 347 | 9 | A: test extraction | LOW |
| `effect_v3/effects/anchor/systems.rs` | 459 | 63 | 396 | 10 | A: test extraction | LOW |
| `effect_v3/effects/second_wind/systems.rs` | 449 | 24 | 425 | 10 | A: test extraction | LOW |
| `effect_v3/effects/chain_lightning/systems.rs` | 445 | 116 | 329 | 6 | A: test extraction | LOW |
| `effect_v3/effects/ramping_damage/config.rs` | 431 | 80 | 351 | 13 | A: test extraction | LOW |
| `effect_v3/walking/walk_effects.rs` | 427 | 160 | 267 | 6 | A: test extraction | LOW |
| `effect_v3/commands/ext.rs` | 425 | 105 | 320 | 12 | A: test extraction | LOW |
| `effect_v3/triggers/node/scan_thresholds.rs` | 410 | 118 | 292 | 9 | A: test extraction | LOW |
| `cells/resources/tests.rs` | 723 | 0 | 723 | 53 | already-extracted, watch | (monitor) |
| `state/.../spawn_cells_from_layout/tests/behaviors.rs` | 590 | 0 | 590 | 11 | already-extracted, watch | (monitor) |
| `state/.../spawn_cells_from_layout/system.rs` | 521 | 521 | 0 | 0 | B: existing todo #10 | (monitor) |
| `cells/definition/tests.rs` | 510 | 0 | 510 | 48 | already-extracted, watch | (monitor) |
| `state/.../spawn_cells_from_layout/tests/helpers.rs` | 447 | 0 | 447 | 0 | helper file, no split | (monitor) |

All file paths are relative to `breaker-game/src/`.

### Priority Guide
- **HIGH**: 1000+ lines, or 800+ test lines (biggest context pollution impact, split immediately, across 2+ files)
- **MEDIUM**: 501-999 lines (noticeable, split at least once)
- **LOW**: 400-500 lines (flag for awareness, will need splitting soon)
- **(monitor)**: Already extracted or covered by existing todo

---

## Refactor Specs

### 1. `evaluate_conditions.rs` (3509 lines, 206 prod / 3303 tests, 64 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/conditions/evaluate_conditions.rs`
- Total lines: 3509 (prod: 206, tests: 3303)
- Strategy: A (test extraction + sub-split into tests/ directory)
- Parent module: `effect_v3/conditions/mod.rs` declares `mod evaluate_conditions;` — re-exports `DuringActive`, `evaluate_condition`, `evaluate_conditions`
- Doc comment: `//! evaluate_conditions -- per-frame condition polling system for During nodes.`
- Target structure:
  ```
  effect_v3/conditions/
    evaluate_conditions/
      mod.rs              // pub(crate) mod system; #[cfg(test)] mod tests;
                          // re-export: pub use system::{DuringActive, evaluate_condition, evaluate_conditions};
                          // re-export: pub(super) use system::{fire_scoped_tree, reverse_scoped_tree, install_armed_entry, reverse_armed_tree};
      system.rs           // lines 1-206 (production code)
      tests/
        mod.rs            // mod helpers; mod during_basic; mod shape_c; mod shape_d;
        helpers.rs        // shared test helpers (tree builders, walk_entity_effects fn, lines ~226-257, ~1049-1109)
        during_basic.rs   // tests for basic During node polling/cycling (lines ~258-1045, ~17 tests)
        shape_c.rs        // Shape C: nested During>When arming/disarming (lines ~1112-1830, ~17 tests)
        shape_d.rs        // Shape D: participant-targeted reversal (lines ~1836-3509, ~30 tests)
  ```
- Test groups (for sub-splitting):
  - `helpers.rs`: during_node_speed_boost, during_shield_damage_boost, during_when_bumped_speed_boost, during_on_bump_bolt_speed_boost, walk_entity_effects (~5 helpers, ~90 lines)
  - `during_basic.rs`: during_fires_inner_effect_when_node_condition_true through installed_during_persists_through_condition_cycling (17 tests, ~790 lines)
  - `shape_c.rs`: shape_c_cond_entering_true_installs_armed_when_entry through shape_c_multiple_entries_with_different_triggers_are_independent (17 tests, ~720 lines)
  - `shape_d.rs`: shape_d_cond_entering_true_installs_armed_on_entry through shape_d_armed_fired_participants_component_is_in_scope + scoped_terminal_fire_converts_to_terminal_fire_with_widened_type (30 tests, ~1675 lines)
- Re-exports needed in `evaluate_conditions/mod.rs`: `pub use system::{DuringActive, evaluate_condition, evaluate_conditions};` (matching what conditions/mod.rs currently re-exports)
- Also re-export private items used by sibling modules: `pub(super) use system::{fire_scoped_tree, reverse_scoped_tree};` if used by armed_source.rs
- Delegate: writer-code can execute this refactor directly

---

### 2. `triggers/bump/bridges.rs` (2718 lines, 311 prod / 2407 tests, 52 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/triggers/bump/bridges.rs`
- Total lines: 2718 (prod: 311, tests: 2407)
- Strategy: A (test extraction + sub-split into tests/ directory)
- Parent module: `effect_v3/triggers/bump/mod.rs` declares `pub mod bridges;`
- Doc comment: (check first line)
- Target structure:
  ```
  effect_v3/triggers/bump/
    bridges/
      mod.rs              // pub(crate) mod system; #[cfg(test)] mod tests;
                          // re-export all pub items from system
      system.rs           // lines 1-311 (production code)
      tests/
        mod.rs            // mod helpers; mod no_bump; mod on_bumped; mod grade_filters; mod occurred; mod staged;
        helpers.rs        // shared test app builder, inject helpers (~lines 313-394)
        no_bump.rs        // on_no_bump_occurred tests (7 tests, ~lines 396-737)
        on_bumped.rs      // on_bumped tests — breaker/bolt walks (11 tests, ~lines 739-1100)
        grade_filters.rs  // on_perfect/early/late_bumped filter tests (12 tests, ~lines 1102-1484)
        occurred.rs       // on_bump/perfect/early/late_occurred broadcast tests (15 tests, ~lines 1485-2080)
        staged.rs         // on_bumped staged entry + nested When tests (5 tests, ~lines 2081-2718)
  ```
- Test groups (for sub-splitting):
  - `helpers.rs`: bump_performed_test_app, inject_bump_performed, and other shared setup (~80 lines)
  - `no_bump.rs`: on_no_bump_occurred_* tests (7 tests, ~340 lines)
  - `on_bumped.rs`: on_bumped_walks_breaker_on_perfect_grade through on_bumped_handles_multiple_messages_per_frame (11 tests, ~360 lines)
  - `grade_filters.rs`: on_perfect_bumped_*, on_early_bumped_*, on_late_bumped_* (12 tests, ~380 lines)
  - `occurred.rs`: on_bump_occurred_*, on_perfect_bump_occurred_*, on_early_bump_occurred_*, on_late_bump_occurred_*, on_bump_whiff_occurred_* (15 tests, ~600 lines)
  - `staged.rs`: on_bumped_staged_entry_*, nested When tests (5 tests, ~640 lines)
- Re-exports needed: all `pub fn` and `pub` items from the production portion of bridges.rs
- Delegate: writer-code can execute this refactor directly

---

### 3. `walking/until.rs` (2206 lines, 227 prod / 1979 tests, 27 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/walking/until.rs`
- Total lines: 2206 (prod: 227, tests: 1979)
- Strategy: A (test extraction + sub-split into tests/ directory)
- Parent module: `effect_v3/walking/mod.rs` declares `mod until;` with re-export `pub use until::{UntilApplied, evaluate_until};`
- Target structure:
  ```
  effect_v3/walking/
    until/
      mod.rs              // pub(crate) mod system; #[cfg(test)] mod tests;
                          // pub use system::{UntilApplied, evaluate_until};
      system.rs           // lines 1-227 (production code)
      tests/
        mod.rs            // mod helpers; mod basic_until; mod until_tracking; mod until_during;
        helpers.rs        // shared test setup functions
        basic_until.rs    // fire/reverse/removal basic behavior (12 tests, ~lines 250-1130)
        until_tracking.rs // UntilApplied component tracking (3 tests, ~lines 1232-1407)
        until_during.rs   // Until with During inner tree composition (12 tests, ~lines 1409-2206)
  ```
- Test groups:
  - `basic_until.rs`: until_fires_inner_fire_effect_on_first_walk through multiple_until_entries_both_reverse_on_respective_gate_triggers (14 tests, ~880 lines)
  - `until_tracking.rs`: until_applied_component_created_on_first_evaluation through until_applied_source_removed_after_reversal (3 tests, ~175 lines)
  - `until_during.rs`: until_with_during_inner_installs_during_on_first_walk through until_with_during_sequence_inner_installs_correctly (10 tests, ~800 lines)
- Re-exports needed: `pub use system::{UntilApplied, evaluate_until};`
- Delegate: writer-code can execute this refactor directly

---

### 4. `stacking/effect_stack.rs` (1049 lines, 81 prod / 968 tests, 57 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/stacking/effect_stack.rs`
- Total lines: 1049 (prod: 81, tests: 968)
- Strategy: A (test extraction + sub-split into tests/ directory)
- Parent module: `effect_v3/stacking/mod.rs` declares `mod effect_stack;` with re-export `pub use effect_stack::EffectStack;`
- Doc comment: `//! EffectStack<T> -- generic stack component for passive effects.`
- Target structure:
  ```
  effect_v3/stacking/
    effect_stack/
      mod.rs              // pub(crate) mod component; #[cfg(test)] mod tests;
                          // pub use component::EffectStack;
      component.rs        // lines 1-81 (production code — the EffectStack<T> struct and impls)
      tests/
        mod.rs            // mod push_pop; mod aggregate; mod multi_source;
        push_pop.rs       // basic push/pop/peek/is_empty/len tests (~20 tests, ~340 lines)
        aggregate.rs      // aggregate computation for each config type (~20 tests, ~340 lines)
        multi_source.rs   // multi-source stacking, reverse_by_source tests (~17 tests, ~280 lines)
  ```
- Re-exports needed: `pub use component::EffectStack;`
- Delegate: writer-code can execute this refactor directly

---

### 5. `effects/circuit_breaker/systems.rs` (1031 lines, 64 prod / 967 tests, 25 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/effects/circuit_breaker/systems.rs`
- Total lines: 1031 (prod: 64, tests: 967)
- Strategy: A (test extraction + sub-split)
- Parent module: `effect_v3/effects/circuit_breaker/mod.rs` declares `pub mod systems;`
- Target structure:
  ```
  effect_v3/effects/circuit_breaker/
    systems/
      mod.rs              // pub(crate) mod system; #[cfg(test)] mod tests;
                          // re-export pub items from system
      system.rs           // lines 1-64 (production code)
      tests/
        mod.rs            // mod helpers; mod counting; mod firing;
        helpers.rs        // test app builder, tick helpers
        counting.rs       // bump counting and decrement tests (~12 tests, ~480 lines)
        firing.rs         // fire dispatch and spawn tests (~13 tests, ~480 lines)
  ```
- Delegate: writer-code can execute this refactor directly

---

### 6. `effects/piercing_beam/config.rs` (1017 lines, 72 prod / 945 tests, 29 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/effects/piercing_beam/config.rs`
- Total lines: 1017 (prod: 72, tests: 945)
- Strategy: A (test extraction + sub-split)
- Parent module: `effect_v3/effects/piercing_beam/mod.rs` declares `pub mod config;` with re-export `pub use config::PiercingBeamConfig;`
- Target structure:
  ```
  effect_v3/effects/piercing_beam/
    config/
      mod.rs              // pub(crate) mod config_impl; #[cfg(test)] mod tests;
                          // pub use config_impl::PiercingBeamConfig;
      config_impl.rs      // lines 1-72 (production code)
      tests/
        mod.rs            // mod helpers; mod fire_tests; mod geometry_tests;
        helpers.rs        // test helpers, app setup
        fire_tests.rs     // fire/reverse dispatch tests (~15 tests)
        geometry_tests.rs // beam geometry and cell targeting tests (~14 tests)
  ```
- Delegate: writer-code can execute this refactor directly

---

### 7. `effects/pulse/systems.rs` (1015 lines, 137 prod / 878 tests, 25 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/effects/pulse/systems.rs`
- Total lines: 1015 (prod: 137, tests: 878)
- Strategy: A (test extraction + sub-split)
- Parent module: `effect_v3/effects/pulse/mod.rs` declares `pub mod systems;`
- Target structure:
  ```
  effect_v3/effects/pulse/
    systems/
      mod.rs              // pub(crate) mod system; #[cfg(test)] mod tests;
                          // re-export pub items from system
      system.rs           // lines 1-137 (production code)
      tests/
        mod.rs            // mod helpers; mod tick_pulse; mod source_chip;
        helpers.rs        // emitter_test_app, tick helpers
        tick_pulse.rs     // ring spawning, radius expansion, expiry tests (~18 tests)
        source_chip.rs    // EffectSourceChip propagation tests (~7 tests)
  ```
- Delegate: writer-code can execute this refactor directly

---

### 8. `effects/entropy_engine/systems.rs` (976 lines, 93 prod / 883 tests, 26 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/effects/entropy_engine/systems.rs`
- Total lines: 976 (prod: 93, tests: 883)
- Strategy: A (test extraction + sub-split)
- Parent module: `effect_v3/effects/entropy_engine/mod.rs` declares `pub mod systems;`
- Target structure:
  ```
  effect_v3/effects/entropy_engine/
    systems/
      mod.rs              // pub(crate) mod system; #[cfg(test)] mod tests;
                          // re-export pub items from system
      system.rs           // lines 1-93 (production code)
      tests/
        mod.rs            // mod helpers; mod counting; mod weighted_random;
        helpers.rs        // shared test helpers
        counting.rs       // counter increment/threshold/reset tests (~13 tests)
        weighted_random.rs // weighted pool selection, deterministic rng tests (~13 tests)
  ```
- Delegate: writer-code can execute this refactor directly

---

### 9. `effects/tether_beam/systems.rs` (973 lines, 76 prod / 897 tests, 30 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/effects/tether_beam/systems.rs`
- Total lines: 973 (prod: 76, tests: 897)
- Strategy: A (test extraction + sub-split)
- Parent module: `effect_v3/effects/tether_beam/mod.rs` declares `pub mod systems;`
- Target structure:
  ```
  effect_v3/effects/tether_beam/
    systems/
      mod.rs              // pub(crate) mod system; #[cfg(test)] mod tests;
                          // re-export pub items from system
      system.rs           // lines 1-76 (production code)
      tests/
        mod.rs            // mod helpers; mod tick_tether; mod source_chip;
        helpers.rs        // shared test helpers
        tick_tether.rs    // beam positioning, entity tracking, despawn tests (~22 tests)
        source_chip.rs    // EffectSourceChip propagation tests (~8 tests)
  ```
- Delegate: writer-code can execute this refactor directly

---

### 10. `triggers/death/bridges.rs` (971 lines, 180 prod / 791 tests, 18 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/triggers/death/bridges.rs`
- Total lines: 971 (prod: 180, tests: 791)
- Strategy: A (test extraction — tests under 800 lines, single tests.rs)
- Parent module: `effect_v3/triggers/death/mod.rs` declares `pub mod bridges;`
- Target structure:
  ```
  effect_v3/triggers/death/
    bridges/
      mod.rs              // pub(crate) mod system; #[cfg(test)] mod tests;
                          // re-export pub items from system
      system.rs           // lines 1-180 (production code)
      tests.rs            // lines 181-971 (18 tests, 791 lines — just under sub-split threshold)
  ```
- Delegate: writer-code can execute this refactor directly

---

### MEDIUM Priority Files (501-999 lines)

For these files, Strategy A (test extraction) with a single `tests.rs` is sufficient since test lines are under 800.

**11. `triggers/impact/bridges.rs`** (961 lines, 297 prod / 664 tests, 15 fns)
- Target: `bridges/mod.rs` + `bridges/system.rs` + `bridges/tests.rs`

**12. `effects/pulse/config.rs`** (792 lines, 71 prod / 721 tests, 13 fns)
- Target: `config/mod.rs` + `config/config_impl.rs` + `config/tests.rs`

**13. `effects/shield/config.rs`** (773 lines, 87 prod / 686 tests, 16 fns)
- Target: `config/mod.rs` + `config/config_impl.rs` + `config/tests.rs`

**14. `walking/when.rs`** (733 lines, 49 prod / 684 tests, 14 fns)
- Target: `when/mod.rs` + `when/system.rs` + `when/tests.rs`
- Re-export: `pub use system::evaluate_when;`

**15. `walking/on.rs`** (713 lines, 72 prod / 641 tests, 16 fns)
- Target: `on/mod.rs` + `on/system.rs` + `on/tests.rs`
- Re-export: `pub use system::evaluate_on;`

**16. `effects/tether_beam/config.rs`** (683 lines, 132 prod / 551 tests, 18 fns)
- Target: `config/mod.rs` + `config/config_impl.rs` + `config/tests.rs`

**17. `effects/shockwave/systems.rs`** (629 lines, 80 prod / 549 tests, 18 fns)
- Target: `systems/mod.rs` + `systems/system.rs` + `systems/tests.rs`

**18. `walking/during.rs`** (624 lines, 83 prod / 541 tests, 8 fns)
- Target: `during/mod.rs` + `during/system.rs` + `during/tests.rs`
- Re-export: `pub use system::evaluate_during;`

**19. `effects/chain_lightning/config.rs`** (568 lines, 71 prod / 497 tests, 19 fns)
- Target: `config/mod.rs` + `config/config_impl.rs` + `config/tests.rs`

**20. `triggers/bolt_lost/bridges.rs`** (544 lines, 38 prod / 506 tests, 11 fns)
- Target: `bridges/mod.rs` + `bridges/system.rs` + `bridges/tests.rs`

**21. `walking/once.rs`** (517 lines, 50 prod / 467 tests, 8 fns)
- Target: `once/mod.rs` + `once/system.rs` + `once/tests.rs`
- Re-export: `pub use system::evaluate_once;`

**22. `triggers/node/bridges.rs`** (509 lines, 77 prod / 432 tests, 9 fns)
- Target: `bridges/mod.rs` + `bridges/system.rs` + `bridges/tests.rs`

**23. `spawn_stamp_registry/watchers/tests/bolt.rs`** (810 lines, 0 prod / 810 tests, 22 fns)
- Strategy C: already in tests dir but 810 lines
- Consider splitting into `bolt_basic.rs` and `bolt_advanced.rs` at ~400 lines each
- Low urgency since it's already properly isolated in a test directory

---

## Recommended Batching

All files are in the `breaker-game` crate's `effect_v3` domain. They can be parallelized by subdomain since they don't touch each other's files.

### Batch 1: Walking module (4 files)
- `walking/until.rs` (HIGH)
- `walking/when.rs` (MEDIUM)
- `walking/on.rs` (MEDIUM)
- `walking/during.rs` (MEDIUM)
- `walking/once.rs` (MEDIUM)
- `walking/walk_effects.rs` (LOW)
- Note: `walking/mod.rs` re-exports must be updated for all of these

### Batch 2: Conditions + Stacking + Commands (3 files)
- `conditions/evaluate_conditions.rs` (HIGH)
- `stacking/effect_stack.rs` (HIGH)
- `commands/ext.rs` (LOW)

### Batch 3: Trigger bridges (5 files)
- `triggers/bump/bridges.rs` (HIGH)
- `triggers/death/bridges.rs` (HIGH)
- `triggers/impact/bridges.rs` (MEDIUM)
- `triggers/bolt_lost/bridges.rs` (MEDIUM)
- `triggers/node/bridges.rs` (MEDIUM)
- `triggers/node/scan_thresholds.rs` (LOW)
- `triggers/time/bridges.rs` (LOW)

### Batch 4: Effect configs + systems (10 files)
- `effects/circuit_breaker/systems.rs` (HIGH)
- `effects/piercing_beam/config.rs` (HIGH)
- `effects/pulse/systems.rs` (HIGH)
- `effects/entropy_engine/systems.rs` (HIGH)
- `effects/tether_beam/systems.rs` (HIGH)
- `effects/pulse/config.rs` (MEDIUM)
- `effects/shield/config.rs` (MEDIUM)
- `effects/tether_beam/config.rs` (MEDIUM)
- `effects/shockwave/systems.rs` (MEDIUM)
- `effects/chain_lightning/config.rs` (MEDIUM)

### Batch 5: Storage (1 file)
- `storage/spawn_stamp_registry/watchers/tests/bolt.rs` (MEDIUM)

### Batch 6: Dispatch (1 file)
- `dispatch/reverse_dispatch.rs` (LOW)

Batches 1-5 can run in parallel since they touch different subdirectories. Within each batch, all splits are independent of each other.

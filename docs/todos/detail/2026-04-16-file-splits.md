# File Splits -- Wave 18: new-cell-modifiers

Scan date: 2026-04-16
Focus: `breaker-game/` crate, files changed or grown due to `feature/new-cell-modifiers` branch.

---

## File Length Review

### Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `breaker-game/src/effect_v3/conditions/evaluate_conditions/tests/shape_d.rs` | 1666 | 0 | 1666 | 30 | C: sub-split oversized test file | HIGH |
| `breaker-game/src/cells/definition/tests.rs` | 1371 | 0 | 1371 | 116 | C: sub-split oversized test file | HIGH |
| `breaker-game/src/effect_v3/triggers/impact/bridges/tests.rs` | 1219 | 0 | 1219 | 29 | C: sub-split oversized test file | HIGH |
| `breaker-game/src/effect_v3/triggers/death/bridges/tests.rs` | 1184 | 0 | 1184 | 30 | C: sub-split oversized test file | HIGH |
| `breaker-game/src/effect_v3/walking/until/tests/basic_until.rs` | 992 | 0 | 992 | 14 | C: sub-split oversized test file | HIGH |
| `breaker-game/src/cells/plugin.rs` | 958 | 106 | 852 | 13 | A: test extraction | HIGH |
| `breaker-game/src/effect_v3/walking/until/tests/until_during.rs` | 823 | 0 | 823 | 10 | C: sub-split oversized test file | HIGH |
| `breaker-game/src/cells/resources/tests.rs` | 813 | 0 | 813 | 62 | C: sub-split oversized test file | HIGH |
| `breaker-game/src/shared/death_pipeline/systems/tests/handle_kill_unit.rs` | 771 | 0 | 771 | 30 | monitor | MEDIUM |
| `breaker-game/src/state/run/node/systems/reset_bolt/tests.rs` | 757 | 0 | 757 | 10 | monitor | MEDIUM |
| `breaker-game/src/state/run/chip_select/systems/snapshot_node_highlights/tests.rs` | 752 | 0 | 752 | 18 | monitor | MEDIUM |
| `breaker-game/src/effect_v3/effects/pulse/config/tests.rs` | 728 | 0 | 728 | 16 | monitor | MEDIUM |
| `breaker-game/src/cells/builder/tests/spawn_tests.rs` | 716 | 0 | 716 | 18 | monitor | MEDIUM |
| `breaker-game/src/cells/builder/tests/definition_tests.rs` | 701 | 0 | 701 | 22 | monitor | MEDIUM |
| `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/tests/behaviors.rs` | 637 | 0 | 637 | 12 | monitor | MEDIUM |
| `breaker-game/src/bolt/systems/bolt_lost/tests/lost_detection_tests.rs` | 635 | 0 | 635 | 14 | monitor | MEDIUM |
| `breaker-game/src/bolt/builder/tests/build_tests.rs` | 619 | 0 | 619 | 14 | monitor | MEDIUM |
| `breaker-game/src/effect_v3/conditions/evaluate_conditions/tests/shape_c.rs` | 617 | 0 | 617 | 14 | monitor | MEDIUM |
| `breaker-game/src/effect_v3/effects/tether_beam/systems/tests/tick_tether.rs` | 606 | 0 | 606 | 12 | monitor | MEDIUM |
| `breaker-game/src/cells/behaviors/survival/salvo/tests/fire_survival_turret.rs` | 589 | 0 | 589 | 10 | monitor | MEDIUM |
| `breaker-game/src/effect_v3/effects/shield/config/tests.rs` | 589 | 0 | 589 | 14 | monitor | MEDIUM |
| `breaker-game/src/shared/size/tests.rs` | 577 | 0 | 577 | 16 | monitor | MEDIUM |
| `breaker-game/src/effect_v3/effects/circuit_breaker/systems/tests/firing.rs` | 572 | 0 | 572 | 10 | monitor | MEDIUM |
| `breaker-game/src/bolt/systems/tick_birthing/tests.rs` | 571 | 0 | 571 | 10 | monitor | MEDIUM |
| `breaker-game/src/effect_v3/walking/on/tests.rs` | 556 | 0 | 556 | 10 | monitor | MEDIUM |
| `breaker-game/src/bolt/builder/tests/definition_tests/from_definition.rs` | 551 | 0 | 551 | 14 | monitor | MEDIUM |
| `breaker-game/src/cells/behaviors/survival/salvo/tests/fire_survival_turret.rs` | 524 | 0 | 524 | 10 | monitor | MEDIUM |
| `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs` | 503 | 503 | 0 | 0 | B: concern separation (already has a todo) | LOW |
| `breaker-game/src/effect_v3/walking/during/tests.rs` | 483 | 0 | 483 | 8 | monitor | LOW |
| `breaker-game/src/walls/builder/tests/build_tests.rs` | 480 | 0 | 480 | 10 | monitor | LOW |
| `breaker-game/src/breaker/systems/bump/tests/no_bump.rs` | 478 | 0 | 478 | 10 | monitor | LOW |
| `breaker-game/src/state/run/chip_select/systems/generate_chip_offerings/tests.rs` | 475 | 0 | 475 | 10 | monitor | LOW |
| `breaker-game/src/effect_v3/effects/piercing_beam/config/tests/geometry_tests.rs` | 465 | 0 | 465 | 10 | monitor | LOW |
| `breaker-game/src/state/run/node/definition/tests/layout_validation.rs` | 450 | 0 | 450 | 12 | monitor | LOW |
| `breaker-game/src/effect_v3/effects/shockwave/systems/tests.rs` | 443 | 0 | 443 | 8 | monitor | LOW |
| `breaker-game/src/bolt/systems/bolt_lost/tests/extra_bolt_tests.rs` | 440 | 0 | 440 | 8 | monitor | LOW |
| `breaker-game/src/breaker/builder/tests/optional_methods_tests.rs` | 428 | 0 | 428 | 10 | monitor | LOW |
| `breaker-game/src/cells/behaviors/sequence/tests/group_c.rs` | 426 | 0 | 426 | 10 | monitor | LOW |
| `breaker-game/src/effect_v3/stacking/effect_stack/tests/aggregate.rs` | 426 | 0 | 426 | 10 | monitor | LOW |
| `breaker-game/src/effect_v3/effects/chain_lightning/config/tests.rs` | 419 | 0 | 419 | 8 | monitor | LOW |
| `breaker-game/src/cells/builder/tests/optional_tests/guarded_builder.rs` | 417 | 0 | 417 | 10 | monitor | LOW |
| `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/tests/helpers.rs` | 417 | 0 | 417 | 0 | monitor (pure helpers) | LOW |
| `breaker-game/src/effect_v3/walking/once/tests.rs` | 412 | 0 | 412 | 6 | monitor | LOW |
| `breaker-game/src/effect_v3/triggers/bump/bridges/tests/staged.rs` | 410 | 0 | 410 | 8 | monitor | LOW |
| `breaker-game/src/cells/builder/core/transitions.rs` | 409 | ~230 | ~179 | 0 | monitor (cfg(test) convenience methods) | LOW |
| `breaker-game/src/bolt/builder/tests/optional_methods_tests.rs` | 402 | 0 | 402 | 10 | monitor | LOW |
| `breaker-game/src/effect_v3/effects/pulse/systems/tests/tick_pulse.rs` | 401 | 0 | 401 | 8 | monitor | LOW |
| `breaker-game/src/effect_v3/triggers/bolt_lost/bridges/tests.rs` | 405 | 0 | 405 | 8 | monitor | LOW |

### Priority Guide
- **HIGH**: 800+ lines, or production file with 60%+ tests -- biggest context pollution impact, split immediately
- **MEDIUM**: 501-799 lines -- noticeable, split if grows past 800
- **LOW**: 400-500 lines -- flag for awareness, will need splitting soon

---

## Refactor Specs

### 1. `cells/definition/tests.rs` (1371 lines, 116 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/cells/definition/tests.rs`
- Total lines: 1371 (prod: 0, tests: 1371)
- Strategy: C (already-extracted test file, oversized)
- Target structure:
  ```
  breaker-game/src/cells/definition/
    mod.rs            // UNCHANGED
    data.rs           // UNCHANGED
    tests/
      mod.rs          // mod helpers; mod toughness; mod definition_with_toughness; mod guarded_behavior; mod validation; mod deserialization; mod volatile_validation; mod magnetic_validation; mod survival;
      helpers.rs      // valid_definition(), valid_guarded_behavior() (shared helpers, lines 1-27)
      toughness.rs    // Part A: Toughness Enum (lines 29-134, ~10 tests)
      definition_with_toughness.rs  // Part B: CellTypeDefinition with toughness (lines 135-209, ~4 tests)
      guarded_behavior.rs           // Part C: GuardedBehavior + slide_speed + delegation (lines 210-406, ~17 tests)
      deserialization.rs            // CellTypeDefinition deserialization + alias (lines 407-467, ~4 tests)
      regen_validation.rs           // Regen rate validation (lines 468-520, ~7 tests)
      volatile_validation.rs        // Volatile validation Wave 1 (lines 521-772, ~25 tests)
      magnetic_validation.rs        // Magnetic validation Wave 5 (lines 773-919, ~10 tests)
      survival.rs                   // Parts A+B+C: AttackPattern + Survival + Survival Validation (lines 920-1371, ~40 tests)
  ```
- Test groups:
  - `helpers.rs`: valid_definition, valid_guarded_behavior (0 tests, ~28 lines)
  - `toughness.rs`: toughness_has_three_distinct_variants, toughness_default_is_standard, etc. (10 tests, ~106 lines)
  - `definition_with_toughness.rs`: validate_accepts_toughness_variants, hp_multiplier tests (4 tests, ~75 lines)
  - `guarded_behavior.rs`: guardian_hp_fraction tests, slide_speed tests, delegation (17 tests, ~197 lines)
  - `deserialization.rs`: ron deserialization, alias validation (4 tests, ~61 lines)
  - `regen_validation.rs`: regen rate bounds, NaN, Inf (7 tests, ~53 lines)
  - `volatile_validation.rs`: volatile bomb/explode validation (25 tests, ~252 lines)
  - `magnetic_validation.rs`: magnetic radius/strength validation (10 tests, ~147 lines)
  - `survival.rs`: AttackPattern enum, Survival variant, Survival validation (40 tests, ~452 lines)
- Imports needed: `use super::super::data::*;` (each sub-file goes up through tests/ to definition/)
- Re-exports needed: none (test-only)
- Delegate: writer-code can execute this refactor directly

### 2. `cells/plugin.rs` (958 lines: 106 prod, 852 tests, 13 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/cells/plugin.rs`
- Total lines: 958 (prod: 106, tests: 852)
- Strategy: A (test extraction) + C (sub-split tests since 852 > 800)
- Target structure:
  ```
  breaker-game/src/cells/
    plugin/
      mod.rs          // pub(crate) mod plugin; #[cfg(test)] mod tests; pub(crate) use plugin::CellsPlugin;
      plugin.rs       // production code (lines 1-106)
      tests/
        mod.rs        // mod helpers; mod guardian; mod sequence; mod armored; mod phantom; mod magnetic; mod survival;
        helpers.rs    // cells_plugin_app(), tick_cells(), enqueue helpers, shared resources
        guardian.rs   // plugin_builds + slide_guardian_cells test (~90 lines)
        sequence.rs   // init_sequence_groups, reset_inactive_hp, advance_sequence (3 tests, ~170 lines)
        armored.rs    // check_armor_direction registration + weak_face_passes (2 tests, ~150 lines)
        phantom.rs    // tick_phantom_phase registration + loading control (2 tests, ~90 lines)
        magnetic.rs   // apply_magnetic_fields registration + loading control (2 tests, ~90 lines)
        survival.rs   // suppress_bolt_immune registration + loading control (2 tests, ~80 lines)
  ```
- Test groups:
  - `helpers.rs`: cells_plugin_app, tick_cells, enqueue helpers, PluginTestPending* resources (~80 lines)
  - `guardian.rs`: plugin_builds, slide_guardian_cells (2 tests, ~90 lines)
  - `sequence.rs`: sequence cross-plugin behaviors 30-32 (3 tests, ~170 lines)
  - `armored.rs`: armored cross-plugin behavior 27 + weak face (2 tests, ~150 lines)
  - `phantom.rs`: phantom cross-plugin behavior 43 (2 tests, ~90 lines)
  - `magnetic.rs`: magnetic cross-plugin behavior 32 (2 tests, ~90 lines)
  - `survival.rs`: survival cross-plugin behaviors 55-58 (2 tests, ~80 lines)
- Imports needed: production code uses `bevy::prelude::*` + domain imports; tests need `super::super::plugin::*` + test infrastructure
- Re-exports needed: `pub(crate) use plugin::CellsPlugin;` in mod.rs
- Parent module: `breaker-game/src/cells/mod.rs` declares `pub(crate) mod plugin;` -- no change needed
- External imports: `CellsPlugin` used in `game.rs`, `app.rs`, some test helpers
- Delegate: writer-code can execute this refactor directly

### 3. `effect_v3/conditions/evaluate_conditions/tests/shape_d.rs` (1666 lines, 30 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/conditions/evaluate_conditions/tests/shape_d.rs`
- Total lines: 1666 (prod: 0, tests: 1666)
- Strategy: C (already in tests/ directory, single file too large)
- Target structure:
  ```
  breaker-game/src/effect_v3/conditions/evaluate_conditions/tests/
    shape_d/
      mod.rs              // mod arming; mod trigger_dispatch; mod participant_tracking; mod disarm; mod lifecycle;
      arming.rs           // condition entry, staying true (lines ~1-200, ~5 tests)
      trigger_dispatch.rs // context matching, impact/bump redirects, scoped terminals (lines ~200-600, ~6 tests)
      participant_tracking.rs // armed fire recording, stacking, multiple participants (lines ~600-850, ~4 tests)
      disarm.rs           // disarm/reversal, clearing stacks, edge cases (lines ~850-1300, ~8 tests)
      lifecycle.rs        // full lifecycle, cross-participant, persistence, in-scope (lines ~1300-1666, ~7 tests)
  ```
- Imports needed: `use super::super::super::*;` + helpers from the tests/ mod
- Re-exports needed: none (test-only)
- Parent module: `evaluate_conditions/tests/mod.rs` declares `mod shape_d;` -- no change needed
- Delegate: writer-code can execute this refactor directly

### 4. `effect_v3/triggers/impact/bridges/tests.rs` (1219 lines, 29 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/triggers/impact/bridges/tests.rs`
- Total lines: 1219 (prod: 0, tests: 1219)
- Strategy: C (already-extracted test file, oversized)
- Target structure:
  ```
  breaker-game/src/effect_v3/triggers/impact/bridges/
    tests/
      mod.rs              // mod helpers; mod bolt_impact; mod breaker_impact; mod salvo_impact; mod edge_cases;
      helpers.rs          // bridge_test_app(), impact_occurred_speed_tree(), inject_impacts(), TestImpactMessages (~98 lines)
      bolt_impact.rs      // impact_occurred_any_fires_for_bolt_impact_cell/wall/breaker, specific_kind tests (lines ~100-430, ~9 tests)
      breaker_impact.rs   // breaker_impact_cell, any_fires_once_per_collision (lines ~430-520, ~4 tests)
      edge_cases.rs       // no_bound_effects, empty_bound_effects, false_match, does_not_fire (lines ~473-710, ~5 tests)
      salvo_impact.rs     // all salvo_impact_breaker tests (lines ~711-1219, ~11 tests)
  ```
- Imports needed: `use super::super::*;` + bridge test helpers
- Re-exports needed: none (test-only)
- Parent module: `bridges/mod.rs` declares `#[cfg(test)] mod tests;` -- no change needed
- Delegate: writer-code can execute this refactor directly

### 5. `effect_v3/triggers/death/bridges/tests.rs` (1184 lines, 30 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/triggers/death/bridges/tests.rs`
- Total lines: 1184 (prod: 0, tests: 1184)
- Strategy: C (already-extracted test file, oversized)
- Target structure:
  ```
  breaker-game/src/effect_v3/triggers/death/bridges/
    tests/
      mod.rs              // mod helpers; mod death_occurred; mod killed; mod died_trigger; mod salvo_death;
      helpers.rs          // death_test_app(), helpers, shared types (~179 lines)
      death_occurred.rs   // death_occurred_any fires on cell/bolt/breaker/wall deaths (lines ~180-578, ~10 tests)
      killed.rs           // killed_any fires on killer, no_panic, no killer = none (lines ~389-577, ~6 tests)
      died_trigger.rs     // died trigger on victim, staged entries, consumes entries (lines ~647-825, ~4 tests)
      salvo_death.rs      // salvo victim, killer, killed_salvo, death_occurred_salvo, staged (lines ~826-1184, ~10 tests)
  ```
- Imports needed: `use super::super::*;` + bridge test helpers
- Re-exports needed: none (test-only)
- Parent module: `bridges/mod.rs` declares `#[cfg(test)] mod tests;` -- no change needed
- Delegate: writer-code can execute this refactor directly

### 6. `effect_v3/walking/until/tests/basic_until.rs` (992 lines, 14 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/walking/until/tests/basic_until.rs`
- Total lines: 992 (prod: 0, tests: 992)
- Strategy: C (already in tests/ directory, single file too large)
- Target structure:
  ```
  breaker-game/src/effect_v3/walking/until/tests/
    basic_until/
      mod.rs              // mod fire_and_reverse; mod removal; mod sequence; mod multi_entry;
      fire_and_reverse.rs // first walk fires, subsequent non-matching no-fire, reversal on gate match (lines ~17-300, ~4 tests)
      removal.rs          // removes entry after reversal, bound_effects empty, no effects after removal (lines ~296-655, ~4 tests)
      sequence.rs         // sequence fires all, reverses all, no reverse on non-matching (lines ~356-554, ~3 tests)
      multi_entry.rs      // fire-and-immediately-reverse, other entries unaffected, multiple independent (lines ~655-992, ~3 tests)
  ```
- Imports needed: `use super::super::*;`
- Re-exports needed: none (test-only)
- Parent module: `until/tests/mod.rs` declares `mod basic_until;` -- no change needed
- Delegate: writer-code can execute this refactor directly

### 7. `effect_v3/walking/until/tests/until_during.rs` (823 lines, 10 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect_v3/walking/until/tests/until_during.rs`
- Total lines: 823 (prod: 0, tests: 823)
- Strategy: C (already in tests/ directory, single file too large)
- Target structure:
  ```
  breaker-game/src/effect_v3/walking/until/tests/
    until_during/
      mod.rs              // mod basic; mod sequences; mod conditions;
      basic.rs            // basic until-during fire, reverse, re-arm (lines ~1-300, ~4 tests)
      sequences.rs        // sequence inner effects, during+sequence combos (lines ~300-550, ~3 tests)
      conditions.rs       // condition interactions, multiple conditions, dual until-during (lines ~550-823, ~3 tests)
  ```
- Imports needed: `use super::super::*;`
- Re-exports needed: none (test-only)
- Parent module: `until/tests/mod.rs` declares `mod until_during;` -- no change needed
- Delegate: writer-code can execute this refactor directly

### 8. `cells/resources/tests.rs` (813 lines, 62 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/cells/resources/tests.rs`
- Total lines: 813 (prod: 0, tests: 813)
- Strategy: C (already-extracted test file, oversized)
- Target structure:
  ```
  breaker-game/src/cells/resources/
    tests/
      mod.rs                  // mod cell_config; mod seedable_registry; mod ron_roundtrip; mod toughness_config;
      cell_config.rs          // cell_defaults_width_height, ron_parses, all_cell_type_rons (lines 1-100, ~5 tests)
      seedable_registry.rs    // SeedableRegistry tests (lines 101-259, ~12 tests)
      ron_roundtrip.rs        // Part M: RON content after toughness + volatile round-trip (lines 260-448, ~14 tests)
      toughness_config.rs     // ToughnessConfig validate + resource tests Part D (lines 449-813, ~31 tests)
  ```
- Imports needed: `use super::super::data::*;` + crate imports
- Re-exports needed: none (test-only)
- Parent module: `resources/mod.rs` declares `#[cfg(test)] mod tests;` -- no change needed
- Delegate: writer-code can execute this refactor directly

---

## Batching for Parallel Agents

These splits touch independent crates/directories with no cross-file dependencies:

**Batch 1 — cells domain** (can run in parallel):
- `cells/definition/tests.rs` -> tests/ directory
- `cells/plugin.rs` -> plugin/ directory with tests/
- `cells/resources/tests.rs` -> tests/ directory

**Batch 2 — effect_v3 conditions** (can run in parallel with Batch 1):
- `effect_v3/conditions/evaluate_conditions/tests/shape_d.rs` -> shape_d/ directory

**Batch 3 — effect_v3 triggers** (can run in parallel with Batches 1-2):
- `effect_v3/triggers/impact/bridges/tests.rs` -> tests/ directory
- `effect_v3/triggers/death/bridges/tests.rs` -> tests/ directory

**Batch 4 — effect_v3 walking** (can run in parallel with Batches 1-3):
- `effect_v3/walking/until/tests/basic_until.rs` -> basic_until/ directory
- `effect_v3/walking/until/tests/until_during.rs` -> until_during/ directory

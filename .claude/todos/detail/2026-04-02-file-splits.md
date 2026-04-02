# Split Oversized Files

**Date:** 2026-04-02
**Branch:** feature/breaker-builder-pattern
**Scan:** Wave 6 (full workspace)

## File Length Review

### Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `breaker-game/src/effect/commands/tests/dispatch_initial_effects_tests.rs` | 1571 | 0 | 1571 | 32 | C: sub-split by behavior | HIGH |
| `breaker-game/src/breaker/queries.rs` | 1440 | 245 | 1195 | 35 | A: test extraction + sub-split | HIGH |
| `breaker-game/src/bolt/builder/tests/definition_tests.rs` | 1329 | 0 | 1329 | 48 | C: sub-split by behavior | HIGH |
| `breaker-game/src/debug/hot_reload/systems/propagate_bolt_definition/tests.rs` | 1035 | 0 | 1035 | 20 | C: sub-split by behavior | HIGH |
| `rantzsoft_spatial2d/src/components/tests/velocity_tests.rs` | 966 | 0 | 966 | 72 | C: sub-split by behavior | HIGH |
| `breaker-game/src/breaker/builder/core.rs` | 830 | 830 | 0 | 0 | B: concern separation | HIGH |
| `breaker-game/src/breaker/systems/spawn_breaker/tests/spawn_or_reuse.rs` | 806 | 0 | 806 | 26 | C: sub-split by behavior | HIGH |
| `breaker-game/src/bolt/builder/core.rs` | 782 | 782 | 0 | 0 | B: concern separation | HIGH |
| `breaker-game/src/bolt/builder/tests/visual_tests.rs` | 751 | 0 | 751 | 26 | MEDIUM (approaching split) | MEDIUM |
| `breaker-game/src/shared/size.rs` | 664 | 98 | 565 | 35 | A: test extraction | MEDIUM |
| `breaker-game/src/effect/triggers/impacted/tests/context_entity_tests.rs` | 626 | 0 | 626 | 13 | MEDIUM (approaching split) | MEDIUM |
| `breaker-game/src/bolt/builder/tests/build_tests.rs` | 620 | 0 | 620 | 23 | MEDIUM (approaching split) | MEDIUM |
| `breaker-game/src/effect/triggers/impact/tests/context_entity_tests.rs` | 611 | 0 | 611 | 13 | MEDIUM (approaching split) | MEDIUM |
| `breaker-game/src/bolt/systems/reset_bolt/tests.rs` | 601 | 0 | 601 | 22 | MEDIUM (approaching split) | MEDIUM |
| `breaker-game/src/effect/effects/piercing_beam/tests/fire_tests/geometry_tests.rs` | 527 | 0 | 527 | 19 | MEDIUM | MEDIUM |
| `breaker-game/src/debug/hot_reload/systems/propagate_breaker_changes/tests.rs` | 516 | 0 | 516 | 10 | MEDIUM | MEDIUM |
| `breaker-game/src/effect/core/types/definitions/enums.rs` | 503 | 432 | 71 | 5 | MEDIUM (mostly prod) | MEDIUM |
| `breaker-game/src/bolt/systems/bolt_lost/tests/lost_detection_tests.rs` | 500 | 0 | 500 | 16 | MEDIUM | MEDIUM |
| `breaker-game/src/effect/triggers/evaluate/tests/on_resolution/resolve_entity_targets.rs` | 490 | 0 | 490 | 13 | MEDIUM | MEDIUM |
| `breaker-game/src/bolt/systems/spawn_bolt/tests/migration_tests.rs` | 487 | 0 | 487 | 12 | MEDIUM | MEDIUM |
| `breaker-game/src/screen/chip_select/systems/generate_chip_offerings/tests.rs` | 461 | 0 | 461 | 11 | MEDIUM | LOW |
| `breaker-game/src/effect/triggers/evaluate/tests/on_resolution/resolve_edge_cases.rs` | 453 | 0 | 453 | 9 | MEDIUM | LOW |
| `breaker-game/src/bolt/systems/dispatch_bolt_effects/tests/basic_dispatch.rs` | 452 | 0 | 452 | 13 | MEDIUM | LOW |
| `rantzsoft_spatial2d/src/builder.rs` | 451 | 295 | 156 | 11 | LOW (mostly prod) | LOW |
| `rantzsoft_physics2d/src/quadtree/tree.rs` | 451 | 451 | 0 | 0 | SKIP (single data structure) | LOW |
| `breaker-game/src/effect/effects/anchor/tests/tick_timer_tests.rs` | 440 | 0 | 440 | 14 | LOW | LOW |
| `breaker-scenario-runner/src/runner/execution.rs` | 439 | 439 | 0 | 0 | SKIP (single concern) | LOW |
| `breaker-game/src/effect/effects/tether_beam/tests/fire_tests/fire_basic.rs` | 435 | 0 | 435 | 15 | LOW | LOW |
| `breaker-game/src/effect/effects/spawn_phantom/tests/fire_tests.rs` | 431 | 0 | 431 | 13 | LOW | LOW |
| `rantzsoft_spatial2d/src/components/definitions.rs` | 402 | 402 | 0 | 0 | SKIP (pure type defs) | LOW |
| `breaker-scenario-runner/src/runner/app.rs` | 402 | 402 | 0 | 0 | SKIP (single concern) | LOW |

### mod.rs Violations (code in mod.rs)

| File | Lines | Content | Priority |
|------|-------|---------|----------|
| `breaker-game/src/effect/triggers/evaluate/tests/bound_and_staged/mod.rs` | 142 | 6 helper fns + 1 resource + test_app | MEDIUM |
| `breaker-game/src/effect/effects/attraction/tests/mod.rs` | 114 | 5 helper fns + 1 resource + 2 test_apps | MEDIUM |
| `breaker-game/src/effect/effects/shockwave/tests/mod.rs` | 96 | 5 helper fns + 1 resource + 2 test_apps | MEDIUM |
| `breaker-game/src/effect/effects/pulse/tests/mod.rs` | 85 | 5 helper fns + 1 resource + 2 test_apps | MEDIUM |
| `breaker-game/src/effect/effects/piercing_beam/tests/mod.rs` | 76 | 5 helper fns + 1 resource + 2 test_apps | LOW |
| `breaker-game/src/effect/triggers/until/tests/mod.rs` | 25 | 2 helper fns | LOW |

### Priority Guide
- **HIGH**: 800+ lines, or 800+ test lines (biggest context pollution impact, split immediately, across 2+ files)
- **MEDIUM**: 400-799 lines (noticeable, split at least once)
- **LOW**: 400-500 lines (flag for awareness, will need splitting soon)

---

## Refactor Specs (HIGH Priority)

---

### 1. `effect/commands/tests/dispatch_initial_effects_tests.rs` (1571 lines, 32 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/effect/commands/tests/dispatch_initial_effects_tests.rs`
- Total lines: 1571 (prod: 0, tests: 1571)
- Strategy: C (oversized test file, already extracted)
- Target structure:
  ```
  breaker-game/src/effect/commands/tests/
    dispatch_initial_effects_tests/
      mod.rs                    // mod helpers; mod breaker_target; mod bolt_target; mod noop_targets; mod deferred_targets; mod source_chip; mod mixed_and_edge_cases; mod component_insertion;
      helpers.rs                // shared test setup (extract common patterns from test bodies)
      breaker_target.rs         // 4 tests: breaker_target_do_effect_fires_immediately, breaker_target_multiple_bare_do_children_all_fire, breaker_target_when_effect_pushes_to_bound_effects, breaker_target_mixed_do_and_when_fires_do_stores_when (~200 lines)
      bolt_target.rs            // 5 tests: bolt_target_dispatches_to_primary_bolt_only, bolt_target_no_primary_bolt_but_breaker_still_processed, bolt_target_bare_do_fires_on_primary_bolt, bolt_target_do_fires_on_only_bolt_when_it_is_primary, bolt_target_do_no_bolts_alongside_breaker (~250 lines)
      noop_targets.rs           // 4 tests: cell_target_is_noop..., cell_target_when_children_noop..., wall_target_is_noop..., wall_target_do_children_noop... (~250 lines)
      deferred_targets.rs       // 6 tests: all_bolts_target_deferred..., all_bolts_do_children..., all_cells_target_deferred..., all_cells_multiple_children..., all_walls_target_deferred..., all_walls_empty_then... (~400 lines)
      no_breaker.rs             // 3 tests: no_breaker_all_bolts..., no_breaker_all_cells..., no_breaker_all_walls... (~150 lines)
      source_chip.rs            // 4 tests: source_chip_some_passes..., source_chip_special_characters..., source_chip_none..., source_chip_some_empty_string... (~170 lines)
      mixed_and_edge_cases.rs   // 4 tests: empty_effects_list..., on_with_empty_then..., multiple_root_effects..., three_root_effects... (~250 lines)
      component_insertion.rs    // 2 tests: bound_effects_and_staged_effects_inserted_if_absent, prior_bound_effects_preserved_new_entry_appended (~100 lines)
  ```
- Parent mod.rs: `breaker-game/src/effect/commands/tests/mod.rs` -- currently declares `mod dispatch_initial_effects_tests;`, unchanged
- Re-exports needed: none (test file)
- Delegate: writer-code can execute this refactor directly

---

### 2. `breaker/queries.rs` (1440 lines, 245 prod / 1195 tests, 35 test fns)

**Refactor spec hint:**
- Source file: `breaker-game/src/breaker/queries.rs`
- Total lines: 1440 (prod: 245, tests: 1195)
- Strategy: A (test extraction + sub-split)
- Target structure:
  ```
  breaker-game/src/breaker/
    queries/
      mod.rs                        // pub(crate) mod data; #[cfg(test)] mod tests; + re-exports
      data.rs                       // production code (lines 1-245: QueryData structs)
      tests/
        mod.rs                      // mod helpers; mod collision_data; mod size_data; mod movement_data; mod dash_data; mod bump_data; mod reset_data; mod scale_data; mod importable;
        helpers.rs                  // test_app(), assert_query_matched(), tick() (~30 lines)
        collision_data.rs           // 4 tests: collision queries (~130 lines)
        size_data.rs                // 2 tests: size queries (~70 lines)
        movement_data.rs            // 5 tests: movement queries (~210 lines)
        dash_data.rs                // 5 tests: dash queries (~300 lines)
        bump_data.rs                // 6 tests: bump timing + grading queries (~230 lines)
        reset_data.rs               // 4 tests: reset queries (~200 lines)
        scale_data.rs               // 3 tests: sync_breaker_scale queries (~120 lines)
        importable.rs               // 2 tests: all_querydata_structs_importable..., telemetry_data_importable... (~80 lines)
  ```
- Parent module: `breaker-game/src/breaker/mod.rs` declares `pub(crate) mod queries;` -- unchanged
- External imports: grep for `use super::super::queries` or `crate::breaker::queries` to verify
- Re-exports needed in mod.rs: all `pub(crate)` QueryData structs from data.rs
- Delegate: writer-code can execute this refactor directly

---

### 3. `bolt/builder/tests/definition_tests.rs` (1329 lines, 48 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/bolt/builder/tests/definition_tests.rs`
- Total lines: 1329 (prod: 0, tests: 1329)
- Strategy: C (oversized test file, already extracted)
- Target structure:
  ```
  breaker-game/src/bolt/builder/tests/
    definition_tests/
      mod.rs                        // mod helpers; mod from_definition; mod with_methods; mod with_overrides; mod ordering;
      helpers.rs                    // make_bolt_definition() helper (~25 lines)
      from_definition.rs            // 14 tests: from_definition_* tests (~430 lines)
      with_methods.rs               // 12 tests: with_base_damage/definition_name/angle_spread/spawn_offset_y solo tests (~350 lines)
      with_overrides.rs             // 12 tests: *_overrides_definition_* tests + all_four combined (~350 lines)
      ordering.rs                   // 10 tests: *_available_in_initial_state, *_before_definition_*, *_before_and_after_* (~200 lines)
  ```
- Parent mod.rs: `breaker-game/src/bolt/builder/tests/mod.rs` declares `mod definition_tests;` -- unchanged
- Re-exports needed: none (test file)
- Delegate: writer-code can execute this refactor directly

---

### 4. `debug/hot_reload/systems/propagate_bolt_definition/tests.rs` (1035 lines, 20 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/debug/hot_reload/systems/propagate_bolt_definition/tests.rs`
- Total lines: 1035 (prod: 0, tests: 1035)
- Strategy: C (oversized test file, already extracted)
- Target structure:
  ```
  breaker-game/src/debug/hot_reload/systems/propagate_bolt_definition/
    tests/
      mod.rs                        // mod helpers; mod restamping; mod skip_conditions; mod multi_entity; mod bound_effects;
      helpers.rs                    // make_bolt_def(), test_app(), seed_and_flush(), mutate_registry() (~50 lines)
      restamping.rs                 // 8 tests: registry_change_restamps_* (base_speed, min/max_speed, radius, base_damage, angle_clamping, zero variants) (~350 lines)
      skip_conditions.rs            // 4 tests: system_does_not_run_on_registry_added, does_not_run_subsequent_frame, handles_zero_entities, skips_entities_without_ref (~200 lines)
      multi_entity.rs               // 3 tests: updates_all_bolt_entities, updates_bolts_with_different_definitions, all_definition_derived_components_restamped (~350 lines)
      bound_effects.rs              // 3 tests: seed_two_effect_bolt helper + hot_reload_rebuilds..., hot_reload_empty..., hot_reload_no_chip... (~250 lines)
  ```
- Parent mod.rs: `breaker-game/src/debug/hot_reload/systems/propagate_bolt_definition/mod.rs` declares `#[cfg(test)] mod tests;` -- unchanged
- Re-exports needed: none (test file)
- Delegate: writer-code can execute this refactor directly

---

### 5. `rantzsoft_spatial2d/src/components/tests/velocity_tests.rs` (966 lines, 72 tests)

**Refactor spec hint:**
- Source file: `rantzsoft_spatial2d/src/components/tests/velocity_tests.rs`
- Total lines: 966 (prod: 0, tests: 966)
- Strategy: C (oversized test file, already extracted)
- Target structure:
  ```
  rantzsoft_spatial2d/src/components/tests/
    velocity_tests/
      mod.rs                        // mod basics; mod arithmetic; mod clamp_angle; mod clamp_speed; mod with_speed; mod from_angle_up; mod rotate_by; mod constrained;
      basics.rs                     // 3 tests: marker_is_component, default_is_zero, speed_returns_magnitude, speed_zero (~30 lines)
      arithmetic.rs                 // 4 tests: add_vec2, sub_vec2, mul_f32, div_f32 + previous_velocity_default (~40 lines)
      clamp_angle.rs                // 20 tests: velocity_clamp_angle_* (shallow, steep, within_bounds, boundary, quadrant, horizontal, vertical, overlapping) (~450 lines)
      clamp_speed.rs                // 5 tests: clamp_high, clamp_low, within_bounds, zero, preserves_direction, exactly_at_min/max (~100 lines)
      with_speed.rs                 // 5 tests: with_speed_* (sets_magnitude, preserves_direction, zero_velocity, negative_direction, zero_produces_zero) (~80 lines)
      from_angle_up.rs              // 8 tests: from_angle_up_* (zero, positive, negative, pi, half_pi, etc.) (~80 lines)
      rotate_by.rs                  // 10 tests: rotate_by_* (zero, preserves_speed, positive, negative, small_angle, pi, diagonal) + clamp_angle near vertical/horizontal (~130 lines)
      constrained.rs                // 7 tests: constrained_* (base_speed, min, max, no_min_max, angle_clamping, zero, all_params) (~80 lines)
  ```
- Parent mod.rs: `rantzsoft_spatial2d/src/components/tests/mod.rs` declares `mod velocity_tests;` -- unchanged
- Re-exports needed: none (test file)
- Delegate: writer-code can execute this refactor directly

---

### 6. `breaker/builder/core.rs` (830 lines, all production)

**Refactor spec hint:**
- Source file: `breaker-game/src/breaker/builder/core.rs`
- Total lines: 830 (prod: 830, tests: 0)
- Strategy: B (concern separation -- typestate types vs builder impl vs terminal methods)
- Target structure:
  ```
  breaker-game/src/breaker/builder/
    core/
      mod.rs                        // pub(crate) mod types; pub(crate) mod transitions; pub(crate) mod terminal; + re-exports
      types.rs                      // Typestate markers (NoDimensions/HasDimensions/...), settings structs (MovementSettings/DashSettings/...), BreakerBuilder struct, optional data struct (~225 lines, lines 34-236)
      transitions.rs                // Entry point (Breaker::builder()), all dimension transitions, visual/role transitions, definition(), optional chainable methods, override methods, private helpers (~560 lines, lines 236-787)
      terminal.rs                   // build() and spawn() terminal impls for all typestate combinations (~140 lines, lines 787-830+)
  ```
- Parent mod.rs: `breaker-game/src/breaker/builder/mod.rs` declares `pub(crate) mod core;` -- unchanged
- Re-exports needed in core/mod.rs: all pub types (NoDimensions, HasDimensions, NoMovement, HasMovement, NoDashing, HasDashing, NoSpread, HasSpread, NoBump, HasBump, Unvisual, Rendered, Headless, NoRole, Primary, Extra, MovementSettings, DashSettings, DashParams, BrakeParams, SettleParams, BumpSettings, BumpFeedbackSettings, BreakerBuilder)
- External imports: check `crate::breaker::builder::core::` usage
- Doc comment: copy `//!` doc comment to mod.rs
- Delegate: writer-code can execute this refactor directly

---

### 7. `breaker/systems/spawn_breaker/tests/spawn_or_reuse.rs` (806 lines, 26 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/breaker/systems/spawn_breaker/tests/spawn_or_reuse.rs`
- Total lines: 806 (prod: 0, tests: 806)
- Strategy: C (oversized test file, already extracted)
- Target structure:
  ```
  breaker-game/src/breaker/systems/spawn_breaker/tests/
    spawn_or_reuse/
      mod.rs                        // mod first_spawn_core; mod first_spawn_components; mod reuse_and_edge_cases;
      first_spawn_core.rs           // 10 tests: first_node_spawns..., has_primary_marker, has_initialized_marker, has_cleanup, has_lives_count, has_position2d, has_scale2d, has_max_speed, has_reflection_spread, sends_spawned_message (~280 lines)
      first_spawn_components.rs     // 9 tests: has_rendered_components, only_selected_definition, has_movement_components, has_dash_tilt_components, has_bump_components, has_size_constraint, has_aabb2d, has_collision_layers, has_default_dynamic_state (~350 lines)
      reuse_and_edge_cases.rs       // 7 tests: previous_position_matches, empty_effects, existing_breaker_reused, existing_sends_spawned, two_existing_preserved, selected_not_in_registry, chrono_breaker_infinite_lives (~250 lines)
  ```
- Parent mod.rs: `breaker-game/src/breaker/systems/spawn_breaker/tests/mod.rs` declares `mod spawn_or_reuse;` -- unchanged
- Re-exports needed: none (test file)
- Delegate: writer-code can execute this refactor directly

---

### 8. `bolt/builder/core.rs` (782 lines, all production)

**Refactor spec hint:**
- Source file: `breaker-game/src/bolt/builder/core.rs`
- Total lines: 782 (prod: 782, tests: 0)
- Strategy: B (concern separation -- typestate types vs builder impl vs terminal methods)
- Target structure:
  ```
  breaker-game/src/bolt/builder/
    core/
      mod.rs                        // pub(crate) mod types; pub(crate) mod transitions; pub(crate) mod terminal; + re-exports
      types.rs                      // Typestate markers, visual markers, optional data struct, BoltBuilder struct (~115 lines, lines 38-127)
      transitions.rs                // Entry point (Bolt::builder()), position/speed/angle/motion/role/visual transitions, from_definition(), optional chainable methods, private helpers (~390 lines, lines 127-530)
      terminal.rs                   // All headless + rendered build()/spawn() terminal impls (~280 lines, lines 530-782)
  ```
- Parent mod.rs: `breaker-game/src/bolt/builder/mod.rs` declares `pub(crate) mod core;` -- unchanged
- Re-exports needed in core/mod.rs: all pub types (NoPosition, HasPosition, NoSpeed, HasSpeed, NoAngle, HasAngle, NoMotion, Serving, HasVelocity, NoRole, Primary, Extra, Unvisual, Rendered, Headless, BoltBuilder)
- External imports: check `crate::bolt::builder::core::` usage
- Doc comment: copy `//!` doc comment to mod.rs
- Delegate: writer-code can execute this refactor directly

---

## Refactor Specs (MEDIUM Priority -- brief)

### 9. `shared/size.rs` (664 lines, 98 prod / 565 tests, 35 test fns)

- Strategy: A (test extraction)
- Target: `shared/size/mod.rs` + `shared/size/types.rs` + `shared/size/tests.rs`
- tests.rs under 800 lines so no sub-split needed

### 10. `effect/triggers/impacted/tests/context_entity_tests.rs` (626 lines, 13 tests)

- Approaching but below 800 line sub-split threshold
- Monitor for growth; no action now

### 11. `effect/triggers/impact/tests/context_entity_tests.rs` (611 lines, 13 tests)

- Approaching but below 800 line sub-split threshold
- Monitor for growth; no action now

### 12-16. Other MEDIUM files (500-751 lines)

- All below 800 line sub-split threshold
- Monitor for growth

---

## mod.rs Violations (MEDIUM Priority)

All 6 mod.rs files contain test helper functions (test_app builders, helper structs, utility fns). Extract helpers to `helpers.rs` in each directory.

### `effect/triggers/evaluate/tests/bound_and_staged/mod.rs` (142 lines)

- Extract to: `bound_and_staged/helpers.rs` (all helper fns + Snapshot resource + test_app)
- mod.rs becomes: `mod helpers; mod bound_effects; mod remove_chains; mod source_chip; mod staged_effects;`

### `effect/effects/attraction/tests/mod.rs` (114 lines)

- Extract to: `attraction/tests/helpers.rs` (test_app, test_app_with_manage, enter_playing, populate_quadtree, TestImpactMessages, enqueue_messages, tick, spatial_params)
- mod.rs becomes: `mod helpers; mod apply_tests; mod fire_tests; mod manage_tests;`

### `effect/effects/shockwave/tests/mod.rs` (96 lines)

- Extract to: `shockwave/tests/helpers.rs` (DamageCellCollector, collect_damage_cells, test_app, enter_playing, damage_test_app, tick, spawn_test_cell, spawn_shockwave)
- mod.rs becomes: `mod helpers; mod damage_tests; mod expansion_tests; mod fire_tests;`

### `effect/effects/pulse/tests/mod.rs` (85 lines)

- Extract to: `pulse/tests/helpers.rs` (DamageCellCollector, collect_damage_cells, test_app, enter_playing, damage_test_app, tick, spawn_test_cell)
- mod.rs becomes: `mod helpers; mod damage_tests; mod fire_tests; mod tick_tests;`

### `effect/effects/piercing_beam/tests/mod.rs` (76 lines)

- Extract to: `piercing_beam/tests/helpers.rs` (DamageCellCollector, collect_damage_cells, piercing_beam_fire_world, tick, spawn_test_cell, piercing_beam_damage_test_app)
- mod.rs becomes: `mod helpers; mod fire_tests; mod process_tests;`

### `effect/triggers/until/tests/mod.rs` (25 lines)

- Extract to: `until/tests/helpers.rs` (test_app, tick)
- mod.rs becomes: `mod helpers; mod desugaring; mod non_until_and_source_chip; mod overclock_pattern;`

---

## Batching for Parallel Writer-Code Agents

Files are grouped by crate to avoid conflicts. Within a crate, files in different domains can run in parallel.

### Batch 1 (rantzsoft_spatial2d crate -- independent)
- velocity_tests.rs sub-split (Strategy C)

### Batch 2 (breaker-game: bolt domain)
- bolt/builder/core.rs concern separation (Strategy B)
- bolt/builder/tests/definition_tests.rs sub-split (Strategy C)

### Batch 3 (breaker-game: breaker domain)
- breaker/queries.rs test extraction + sub-split (Strategy A)
- breaker/builder/core.rs concern separation (Strategy B)
- breaker/systems/spawn_breaker/tests/spawn_or_reuse.rs sub-split (Strategy C)

### Batch 4 (breaker-game: effect domain)
- effect/commands/tests/dispatch_initial_effects_tests.rs sub-split (Strategy C)

### Batch 5 (breaker-game: debug domain)
- debug/hot_reload/systems/propagate_bolt_definition/tests.rs sub-split (Strategy C)

### Batch 6 (breaker-game: mod.rs violations -- all independent)
- All 6 mod.rs helper extractions (can run in parallel since they touch different directories)

### Batch 7 (breaker-game: shared domain)
- shared/size.rs test extraction (Strategy A)

**Note:** Batches 1, 4, and 5 have zero overlap with each other or with batches 2-3. Batches 2 and 3 are in the same crate but different domains (bolt vs breaker) so can run in parallel. Batch 6 (mod.rs fixes) can run in parallel with everything since each touches a unique directory.

**Recommended execution order:** Run batches 1-5 + batch 6 all in parallel (7 agents). Then batch 7. Then Basic Verification Tier.

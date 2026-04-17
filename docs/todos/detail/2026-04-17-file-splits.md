# File Splits: Heal Pipeline commit

Scan date: 2026-04-17
Branch: develop (post heal-pipeline merge)
Scope: `breaker-game/src/shared/death_pipeline/` plus workspace-wide awareness scan.

The heal-pipeline spec explicitly acknowledged two oversized files to "address in a follow-up split commit (Strategy A)". This todo covers those two splits. It also surfaces the other ≥400-line files already present in the workspace (most were previously flagged in earlier file-splits detail files; see the "Pre-existing oversized files" section).

## File Length Review

### Heal Pipeline oversized files (this commit, actionable)

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `shared/death_pipeline/systems/tests/apply_heal.rs` | 1897 | 0 | 1897 | 60 | C: already-extracted, sub-split | HIGH |
| `shared/death_pipeline/plugin.rs` | 1029 | 136 | 893 | 17 | A: test extraction (+ sub-split tests by group) | HIGH |

All paths are relative to `breaker-game/src/`.

### Priority Guide
- **HIGH**: 1000+ lines, or 800+ test lines
- **MEDIUM**: 501-999 lines
- **LOW**: 400-500 lines
- **(monitor)**: Already covered by a prior file-splits todo

---

## Refactor Specs

### 1. `shared/death_pipeline/systems/tests/apply_heal.rs` (1897 lines, 60 tests)

**Refactor spec hint:**
- Source file: `breaker-game/src/shared/death_pipeline/systems/tests/apply_heal.rs`
- Total lines: 1897 (prod: 0 — test-only file, tests: 1897)
- Strategy: C (already-extracted test file, convert `apply_heal.rs` into `apply_heal/` directory and sub-split tests by behavior group)
- Parent module: `shared/death_pipeline/systems/tests/mod.rs` declares `mod apply_heal;` — unchanged (Rust resolves `apply_heal.rs` or `apply_heal/mod.rs`)
- Doc comment to preserve on the new `apply_heal/mod.rs`:
  ```
  //! Tests for `apply_heal<T>`.
  //!
  //! Covers Groups A–H (Behaviors 1–25), plus Behavior 31 (per-T queue isolation
  //! with `Salvo` + `TestEntity` side-by-side) and Behavior 35 (`HealCap`
  //! exhaustive compile-time match) from the heal-pipeline test spec.
  ```
- Target structure:
  ```
  shared/death_pipeline/systems/tests/
    apply_heal/
      mod.rs                   // mod baseline; mod ceiling_clamp; mod mixed_cap;
                               //   mod dead_filter; mod invulnerable_filter;
                               //   mod target_edges; mod numeric_edges;
                               //   mod attribution; mod per_t_isolation;
                               // Plus the module-level doc comment above and the
                               // `HealCap` exhaustive `const _: fn(HealCap) = ...`
                               // compile-time pin from lines 24-30 (Behavior 35).
      baseline.rs              // Group A (Behaviors 1-3, lines ~32-214, 8 tests)
      ceiling_clamp.rs         // Group B (Behaviors 4-9, lines ~215-650, 13 tests)
      mixed_cap.rs             // Group C (Behaviors 10-11, lines ~651-894, 7 tests)
      dead_filter.rs           // Group D (Behaviors 12-14, lines ~895-1022, 6 tests)
      invulnerable_filter.rs   // Group E (Behaviors 15-16, lines ~1023-1322, 9 tests)
      target_edges.rs          // Group F (Behaviors 17-19, lines ~1323-1382, 3 tests)
      numeric_edges.rs         // Group G (Behaviors 21-23, lines ~1383-1745, 13 tests)
      attribution.rs           // Group H (Behaviors 24-25, lines ~1746-1817, 2 tests)
      per_t_isolation.rs       // Behavior 31 (lines ~1818-1897, 2 tests)
  ```
- Test groups (for sub-splitting), verbatim from the source:
  - `baseline.rs`: heal_increases_hp_current_by_amount, heal_lands_exactly_on_ceiling_starting_fallback, multiple_heals_same_tick_accumulate_same_cap, multiple_heals_clamp_engages_across_messages, heal_max_cap_falls_back_to_starting_when_max_is_none, heal_at_starting_no_overflow_when_max_none (+ Group A tail) — 8 tests, ~180 lines
  - `ceiling_clamp.rs`: heal_max_cap_clamps_at_hp_max_when_some, heal_max_cap_at_max_is_noop, heal_max_cap_huge_amount_falls_back_to_starting, heal_max_cap_with_max_below_starting_honours_max, heal_max_cap_at_depressed_max_is_noop, heal_starting_cap_clamps_at_starting_ignoring_higher_max, heal_starting_cap_at_starting_is_noop, heal_starting_cap_over_starting_never_lowers_hp, heal_starting_cap_equals_max_cap_when_max_none, heal_starting_cap_additive_when_under_ceiling, heal_cap_variants_identical_when_max_none, heal_max_cap_over_cap_pre_state_never_lowers_hp, heal_starting_cap_over_cap_pre_state_never_lowers_hp — 13 tests, ~435 lines
  - `mixed_cap.rs`: mixed_cap_starting_then_max_second_ceiling_extends, mixed_cap_max_then_starting_second_message_is_noop, mixed_cap_small_additive_both_under_ceiling, per_message_ceiling_re_read_not_cached, per_message_ceiling_re_read_order_matters, mixed_cap_both_under_lower_ceiling_accumulate, mixed_cap_both_under_lower_ceiling_reversed_order — 7 tests, ~244 lines
  - `dead_filter.rs`: heal_on_dead_entity_is_noop_max_cap, heal_on_dead_negative_hp_entity_is_noop, heal_on_dead_entity_is_noop_starting_cap, heal_on_dead_entity_does_not_remove_dead_marker, heal_applies_to_non_dead_entity_with_zero_hp, heal_on_dead_entity_does_not_set_killed_by — 6 tests, ~128 lines
  - `invulnerable_filter.rs`: heal_on_invulnerable_entity_is_noop_max_cap, heal_on_invulnerable_entity_is_noop_starting_cap, heal_on_invulnerable_entity_preserves_marker, heal_on_dead_and_invulnerable_entity_is_noop_max_cap, heal_on_dead_and_invulnerable_entity_is_noop_starting_cap, heal_on_dead_and_invulnerable_entity_preserves_both_markers, mixed_batch_invulnerable_vs_vulnerable, mixed_batch_invulnerable_vs_vulnerable_reversed_order, mixed_batch_invulnerable_starting_vulnerable_max — 9 tests, ~300 lines
  - `target_edges.rs`: heal_targeting_entity_without_marker_is_skipped, heal_targeting_entity_without_hp_does_not_panic, heal_targeting_despawned_entity_does_not_panic — 3 tests, ~60 lines
  - `numeric_edges.rs`: heal_with_zero_amount_is_noop_max_cap, heal_with_zero_amount_is_noop_starting_cap, heal_with_negative_amount_is_noop_max_cap, heal_with_large_negative_amount_is_noop, heal_with_negative_zero_amount_is_noop, heal_with_negative_amount_is_noop_starting_cap, heal_with_nan_amount_does_not_corrupt_hp_max_cap, heal_with_nan_amount_does_not_corrupt_hp_starting_cap, heal_with_infinity_max_cap_no_max_clamps_at_starting, heal_with_infinity_starting_cap_ignores_max, heal_with_infinity_max_cap_with_max_clamps_at_max, heal_with_neg_infinity_is_noop — 13 tests (numbered 21-23 in spec), ~363 lines
  - `attribution.rs`: heal_does_not_write_to_killed_by, heal_payload_fields_do_not_leak_into_other_components — 2 tests, ~72 lines
  - `per_t_isolation.rs`: heal_wrong_monomorphization_test_entity_message_does_not_heal_salvo, heal_wrong_monomorphization_salvo_message_does_not_heal_test_entity — 2 tests, ~80 lines
- Imports needed in each sub-file (matches current header of `apply_heal.rs`):
  ```rust
  use std::marker::PhantomData;                 // only files that build HealDealt payloads manually

  use bevy::prelude::*;

  use super::super::helpers::{                  // helpers is at tests/helpers.rs — pub(super) visibility still reaches this depth
      PendingHeal, TestEntity, build_apply_heal_app, heal_msg, spawn_test_entity,
      spawn_test_entity_dead,
  };
  use crate::{
      cells::behaviors::survival::salvo::components::Salvo,   // per_t_isolation.rs only
      prelude::*,
      shared::death_pipeline::{
          heal_dealt::{HealCap, HealDealt},
          systems::apply_heal,
      },
  };
  ```
  Each sub-file takes only the imports it actually uses.
- `apply_heal/mod.rs` contents (wiring + the compile-time `HealCap` pin, which is spec-Behavior-35 and belongs at the module root):
  ```rust
  //! Tests for `apply_heal<T>`.
  //!
  //! Covers Groups A–H (Behaviors 1–25), plus Behavior 31 (per-T queue isolation
  //! with `Salvo` + `TestEntity` side-by-side) and Behavior 35 (`HealCap`
  //! exhaustive compile-time match) from the heal-pipeline test spec.

  use crate::shared::death_pipeline::heal_dealt::HealCap;

  // Behavior 35: HealCap exhaustive compile-time match — adding a new variant
  // without updating this match breaks the build.
  const _: fn(HealCap) = |cap| match cap {
      HealCap::Starting | HealCap::Max => (),
  };

  mod attribution;
  mod baseline;
  mod ceiling_clamp;
  mod dead_filter;
  mod invulnerable_filter;
  mod mixed_cap;
  mod numeric_edges;
  mod per_t_isolation;
  mod target_edges;
  ```
- Re-exports needed: none — this is a test module, nothing is publicly re-exported.
- Parent is unchanged: `shared/death_pipeline/systems/tests/mod.rs` still says `mod apply_heal;`.
- Delegate: writer-code can execute this refactor directly.

---

### 2. `shared/death_pipeline/plugin.rs` (1029 lines, 136 prod / 893 tests, 17 test fns + helpers)

**Refactor spec hint:**
- Source file: `breaker-game/src/shared/death_pipeline/plugin.rs`
- Total lines: 1029 (prod: 136, tests: 893)
- Strategy: A (test extraction — convert to `plugin/` directory, move inline `#[cfg(test)] mod tests` into `plugin/tests/`, sub-split tests by group)
- Parent module: `shared/death_pipeline/mod.rs` declares `pub(crate) mod plugin;` and `pub(crate) use plugin::DeathPipelinePlugin;` — both lines remain unchanged (Rust resolves `plugin.rs` or `plugin/mod.rs`).
- Doc comment to preserve: `//! DeathPipelinePlugin — registers the unified damage -> death -> heal -> despawn pipeline.`
- Target structure:
  ```
  shared/death_pipeline/plugin/
    mod.rs                     // pub(crate) mod system;
                               // #[cfg(test)] mod tests;
                               // pub(crate) use system::DeathPipelinePlugin;
                               // Top-level module doc comment stays here.
    system.rs                  // Lines 1-136 verbatim (production code).
                               // `DeathPipelinePlugin` + `impl Plugin for DeathPipelinePlugin`.
    tests/
      mod.rs                   // mod helpers; mod registration; mod wiring; mod pipeline_ordering;
      helpers.rs               // Shared test fixture (see below).
      registration.rs          // Group K / Behavior 32 — HealDealt<T> message registration (5 tests).
      wiring.rs                // Group K / Behavior 33 — apply_heal::<T> wiring per entity type (5 tests).
      pipeline_ordering.rs     // Group I / Behaviors 26–29 — damage/heal same-tick ordering (7 tests).
  ```
- `plugin/tests/helpers.rs` (pub(super) within `plugin/tests/`) — consolidates the shared fixture currently inline in `plugin.rs` lines 139-200 and the per-type enqueue helpers:
  - `PluginTestEntity` struct + `impl GameEntity for PluginTestEntity`
  - `build_pipeline_app() -> App`
  - DeathPipelineSystems exhaustive compile-time match (lines 193-200, Behavior 34) — keep at helpers.rs module scope.
  - Per-type enqueue fns (only those used by registration/wiring/ordering groups): `enqueue_cell_heal`, `enqueue_bolt_heal`, `enqueue_wall_heal`, `enqueue_breaker_heal`, `enqueue_salvo_heal_plugin`, `enqueue_pt_damage`, `enqueue_pt_heal`, `pt_damage_msg`, `pt_heal_msg`.
- Test groups (for sub-splitting):
  - `registration.rs` (Group K / Behavior 32, lines ~202-267 in plugin.rs): plugin_registers_heal_dealt_for_cell, plugin_registers_heal_dealt_for_bolt, plugin_registers_heal_dealt_for_wall, plugin_registers_heal_dealt_for_breaker, plugin_registers_heal_dealt_for_salvo — 5 tests, ~65 lines
  - `wiring.rs` (Group K / Behavior 33, lines ~269-545 in plugin.rs): plugin_registers_apply_heal_cell, plugin_registers_apply_heal_bolt, plugin_registers_apply_heal_wall, plugin_registers_apply_heal_breaker, plugin_registers_apply_heal_salvo (plus the `enqueue_*_heal` helpers they rely on, which move to helpers.rs) — 5 tests, ~275 lines
  - `pipeline_ordering.rs` (Group I / Behaviors 26-29, lines ~546-1029 in plugin.rs): damage_kills_before_heal_runs_max_cap, damage_kills_before_heal_runs_massive_heal_blocked, damage_kills_before_heal_runs_cover_overkill_blocked, damage_kills_before_heal_runs_starting_cap_blocked, heal_after_non_lethal_damage_adds_in_same_tick, heal_after_non_lethal_damage_order_insensitive, sequence_tick_kill_then_heal_is_skipped, apply_heal_runs_after_handle_kill_observationally — 8 tests, ~480 lines

  Note: the single Group K exhaustive `const _: fn(DeathPipelineSystems) = ...` compile-time pin (lines 195-200) stays in `helpers.rs` so all three test sub-modules import it implicitly (const items are evaluated at compile time regardless of use).
- Imports needed in each tests sub-file:
  ```rust
  use bevy::prelude::*;

  // Pulls PluginTestEntity, build_pipeline_app, and the per-type enqueue helpers.
  use super::helpers::*;
  // Pulls DeathPipelinePlugin, the wired systems, and the DeathPipelineSystems set.
  use super::super::system::*;
  use crate::shared::{
      death_pipeline::{
          damage_dealt::DamageDealt,
          dead::Dead,
          destroyed::Destroyed,
          heal_dealt::{HealCap, HealDealt},
          hp::Hp,
          kill_yourself::KillYourself,
          killed_by::KilledBy,
          sets::DeathPipelineSystems,
          systems,
      },
      test_utils::{MessageCollector, attach_message_capture, tick},
  };
  ```
  Each file keeps only the imports it actually uses.
- `plugin/mod.rs` contents:
  ```rust
  //! `DeathPipelinePlugin` — registers the unified damage -> death -> heal -> despawn pipeline.

  pub(crate) mod system;

  #[cfg(test)]
  mod tests;

  pub(crate) use system::DeathPipelinePlugin;
  ```
- `plugin/tests/mod.rs` contents:
  ```rust
  mod helpers;
  mod pipeline_ordering;
  mod registration;
  mod wiring;
  ```
- Re-exports needed: `pub(crate) use system::DeathPipelinePlugin;` — matches the existing `pub(crate) use plugin::DeathPipelinePlugin;` in `shared/death_pipeline/mod.rs`, which remains unchanged.
- Parent is unchanged: `shared/death_pipeline/mod.rs` still says `pub(crate) mod plugin;` and `pub(crate) use plugin::DeathPipelinePlugin;`.
- Delegate: writer-code can execute this refactor directly.

---

## Pre-existing oversized files (workspace-wide awareness scan)

The following `.rs` files across the workspace are also ≥400 lines. **None are new in the heal-pipeline commit** — every file below pre-dates this branch. Most are already covered by a prior file-splits todo (see `docs/todos/detail/2026-04-14-file-splits.md` and earlier). They are listed here for situational awareness only; do not execute splits for them as part of this todo.

### HIGH (≥1000 lines)

| File | Lines | Covered by |
|------|-------|-----------|
| `breaker-scenario-runner/src/coverage.rs` | 954 | (monitor — pre-existing, was not in recent todos) |
| `breaker-scenario-runner/src/main.rs` | 736 | (monitor — pre-existing) |

(Full HIGH list from 2026-04-14 in `effect_v3/**` — those splits are executable via the prior todo, not this one.)

### MEDIUM (500–999 lines) — unchanged from 2026-04-14

Selection from the workspace-wide scan (all ≥500 lines). Bulk of these are `effect_v3/**` and already tracked in `2026-04-14-file-splits.md`:

- `breaker-game/src/shared/death_pipeline/systems/tests/handle_kill_unit.rs` — 770 lines (pre-existing, flag as NEW MEDIUM — was below threshold before; now worth a follow-up)
- `breaker-game/src/shared/size/tests.rs` — 632 lines (already flagged)
- `breaker-game/src/cells/builder/tests/spawn_tests.rs` — 715 lines (already flagged)
- `breaker-game/src/cells/builder/tests/definition_tests.rs` — 700 lines (already flagged)
- `breaker-game/src/state/run/chip_select/systems/spawn_chip_select.rs` — 701 lines (already flagged)
- ...plus the complete list from `docs/todos/detail/2026-04-14-file-splits.md`.

### LOW (400–500 lines) — unchanged

The workspace contains approximately 50 files in the 400–500 range. None are net-new from the heal-pipeline commit. Refer to `docs/todos/detail/2026-04-14-file-splits.md` for the pre-existing inventory.

---

## Batching for parallel writer-code agents

Both actionable splits are inside `breaker-game/src/shared/death_pipeline/`. They touch different files and different test module subtrees (`systems/tests/apply_heal.rs` vs. `plugin.rs`), so they can run **in parallel in the same wave**:

- **Wave 1a:** Split `systems/tests/apply_heal.rs` → `systems/tests/apply_heal/` (Strategy C — 9 sub-files).
- **Wave 1b:** Split `plugin.rs` → `plugin/` directory (Strategy A + tests sub-split — 5 files).

After both complete: run Basic Verification Tier (`cargo all-dclippy` + `cargo all-dtest`) to confirm the tests still compile and pass under the new module layout, then Standard Verification Tier before commit.

# File Splits — 2026-04-21 sweep

Scan date: 2026-04-21
Branch: `feature/protocol-burnout`
Scope: Full-workspace 400-line sweep. Primary focus: files touched on `feature/protocol-burnout` (protocol protocols, invariant checkers, lifecycle mutations). Secondary: any file anywhere over the threshold.

## Summary

All 8 custom-system protocol modules authored on this branch (Burnout, Reckless Dash, Echo Strike, Fission, Iron Curtain, Debt Collector, Siphon, Greed) are directory modules with per-behavior test sub-files. One in-branch file — `echo_strike/tests/on_impact.rs` — has crossed the 800-line HIGH threshold and needs a sub-split.

Outside the branch, one HIGH-adjacent item (`coverage/tests.rs` 802) and a cluster of already-sub-split MEDIUM test files (all previously acknowledged as awareness-only) continue to drift upward. No new MEDIUM+ outside the 800+ set warrants a merge-blocking split.

## File Length Review — actionable items

### Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `breaker-game/src/protocol/protocols/echo_strike/tests/on_impact.rs` | 823 | 0 | 823 | 24 | C: already extracted, sub-split by behavior band | **HIGH** |
| `breaker-scenario-runner/src/coverage/tests.rs` | 802 | 0 | 802 | 28 | C: already extracted, sub-split by behavior group | **HIGH** |
| `breaker-game/src/cells/systems/apply_damage_to_cells/tests/tether_redirect.rs` | 876 | 0 | 876 | 20 | C: already extracted, sub-split | MEDIUM (monitor) |
| `breaker-game/src/protocol/protocols/afterimage/tests/check_phantom_bounce.rs` | 880 | 0 | 880 | 22 | C: already extracted, sub-split | MEDIUM (monitor) |
| `breaker-game/src/state/run/chip_select/systems/handle_chip_input/tests.rs` | 922 | 0 | 922 | 31 | C: already extracted, sub-split | MEDIUM (monitor) |
| `breaker-game/src/state/run/chip_select/systems/generate_chip_offerings/tests.rs` | 884 | 0 | 884 | 24 | C: already extracted, sub-split | MEDIUM (monitor) |
| `breaker-game/src/protocol/protocols/tier_regression/tests/apply.rs` | 887 | 0 | 887 | 22 | C: already extracted, sub-split | MEDIUM (monitor) |

### LOW / awareness — no action this pass

All under 800 lines; all already sub-split into per-behavior files inside a `tests/` directory. Monitor only:

`state/run/node/systems/reset_bolt/tests.rs` 756,
`shared/death_pipeline/systems/tests/handle_kill_unit.rs` 770,
`effect_v3/triggers/bump/bridges/tests/occurred.rs` 752,
`effect_v3/effects/tether_beam/systems/tests/tick_tether.rs` 722,
`hazard/hazards/momentum/tests/split_check.rs` 742,
`runner/tests/streaming_tests.rs` 742,
`bolt/systems/bolt_lost/tests/lost_detection_tests.rs` 720,
`shared/size/tests.rs` 709,
`cells/builder/tests/spawn_tests.rs` 715,
`cells/builder/tests/definition_tests.rs` 700,
`state/run/node/systems/spawn_cells_from_layout/tests/behaviors.rs` 687,
`effect_v3/walking/when/tests.rs` 680,
`effect_v3/conditions/evaluate_conditions/tests/during_basic.rs` 790,
`effect_v3/effects/pulse/config/tests.rs` 727,
`effect_v3/effects/shield/config/tests.rs` 684,
`effect_v3/effects/circuit_breaker/systems/tests/firing.rs` 659,
`stateflow/dispatch/tests.rs` 687,
plus several in the 500–700 band.

### Priority Guide
- **HIGH**: 800+ test lines OR 1000+ total. Two items above this bar → split now.
- **MEDIUM (monitor)**: 800+ total but already structured as a per-behavior sub-file of a `tests/` directory. Split only if they grow further or interfere with refactoring in that area.
- **LOW**: 400–500 lines; no action.

---

## Refactor Specs (HIGH only)

### 1. `breaker-game/src/protocol/protocols/echo_strike/tests/on_impact.rs` (823 lines) — HIGH

**Refactor spec hint:**
- Source file: `breaker-game/src/protocol/protocols/echo_strike/tests/on_impact.rs`
- Total lines: 823 (all tests — the file lives under `tests/`)
- Strategy: C (convert the single test file into a test sub-directory, sub-split by behavior band)
- Parent module: `breaker-game/src/protocol/protocols/echo_strike/tests/mod.rs` declares `mod on_impact;` — unchanged
- Doc comment to preserve on new `on_impact/mod.rs`:
  ```
  //! Group D — `echo_strike_on_impact` (Behaviors 16–33).
  //!
  //! Pins the `BoltImpactCell` consumer: ... (keep full doc block verbatim)
  ```
- Target structure:
  ```
  breaker-game/src/protocol/protocols/echo_strike/tests/on_impact/
    mod.rs                  // doc comment above
                            // mod helpers;
                            // mod config_and_empty;  mod falloff;  mod fifo_dedup;
                            // mod base_damage;       mod sentinel_and_dealer;
                            // mod tolerance_and_isolation;  mod gating;
    helpers.rs              // seed_canonical(app) — pub(super) fn;
                            //   re-export helper alias if needed.
                            //   Lines ~39–42 of the original.
    config_and_empty.rs     // Behaviors 16–19 — no config / empty network / first impact / new cell absent from echo damage
                            //   on_impact_early_returns_and_clears_reader_when_config_absent
                            //   primed_bolt_empty_network_registers_impact_no_damage
                            //   primed_bolt_with_no_network_component_gets_network_on_demand
                            //   primed_impact_one_existing_echo_deals_newest_fraction_pushes_back
                            //   primed_impact_newly_impacted_cell_not_in_echo_damage
    falloff.rs              // Behaviors 20–22 — 2-echo skipping middle, 3-echo full falloff, 2-echo new-cell exclusion
                            //   primed_impact_two_echoes_uses_oldest_and_newest_skipping_middle
                            //   primed_impact_two_echoes_new_cell_not_in_echo_damage
                            //   primed_impact_three_echoes_full_falloff_and_fifo_eviction
    fifo_dedup.rs           // Behaviors 23, 27–29 — FIFO below cap, dedup move-to-back on middle / newest / oldest
                            //   fifo_eviction_does_not_fire_when_below_max_echoes
                            //   primed_impact_on_existing_middle_echo_dedups_and_moves_to_back
                            //   primed_impact_on_existing_newest_echo_is_ordering_no_op
                            //   primed_impact_on_existing_oldest_echo_moves_it_to_back
    base_damage.rs          // Behaviors 24–26 — non-primed variants + zero / default base damage
                            //   non_primed_bolt_impact_no_echo_no_network_change
                            //   non_primed_bolt_impact_with_empty_network_remains_empty
                            //   primed_impact_zero_base_damage_emits_zero_messages_but_registers_echo
                            //   primed_impact_without_bolt_base_damage_uses_default
    sentinel_and_dealer.rs  // Behaviors on message metadata
                            //   every_emitted_echo_damage_message_has_echo_strike_sentinel
                            //   every_emitted_echo_damage_message_has_dealer_set_to_bolt
    tolerance_and_isolation.rs // Despawn + multi-bolt
                            //   impact_for_despawned_bolt_is_tolerated
                            //   primed_impact_does_not_touch_other_bolts_networks
    gating.rs               // Gating / NodeState / same-tick double impact
                            //   on_impact_with_echo_strike_inactive_does_nothing
                            //   on_impact_with_node_state_not_playing_does_nothing
                            //   primed_bolt_receives_two_impacts_same_tick_single_shot
                            //   primed_bolt_two_impacts_same_tick_with_two_existing_echoes
  ```
- Imports per sub-file: `use super::super::super::system::{ECHO_STRIKE_SENTINEL, EchoNetwork, EchoPrimed};` plus `use super::super::super::tests::helpers::{...}` (adjust `super::` count — sub-files are one directory deeper than the original) plus `use super::helpers::seed_canonical;` plus `use crate::{bolt::resources::DEFAULT_BOLT_BASE_DAMAGE, prelude::*};`
- After split, grep `super::{super::system` and `super::helpers` usages to confirm correct depth.
- No change to echo_strike production code, no change to `echo_strike/tests/mod.rs` (still declares `mod on_impact;` — Rust resolves to the directory module).
- Delegate: writer-code can execute directly.

---

### 2. `breaker-scenario-runner/src/coverage/tests.rs` (802 lines) — HIGH

**Refactor spec hint:**
- Source file: `breaker-scenario-runner/src/coverage/tests.rs`
- Total lines: 802 (all tests)
- Strategy: C (convert the single test file into a test sub-directory, sub-split by coverage-check concern)
- Parent module: `breaker-scenario-runner/src/coverage/mod.rs` declares `#[cfg(test)] mod tests;` — unchanged
- Doc comment (none in the original `tests.rs` — add a brief one describing the group on the new `tests/mod.rs`)
- Target structure:
  ```
  breaker-scenario-runner/src/coverage/tests/
    mod.rs                  // mod helpers;
                            // mod self_test_coverage;
                            // mod layout_coverage;
                            // mod invariant_kind_enumeration;
                            // mod formatting;
    helpers.rs              // minimal_scenario(layout, allowed_failures) — pub(super) fn
                            //   Lines ~4–23 of the original.
    self_test_coverage.rs   // Tests that exercise `missing_self_tests` discovery —
                            //   missing_self_test_detection_finds_uncovered_invariants
                            //   + the self-test scenario coverage variants
                            //   (~11 tests in the "self-test" behavior band).
    layout_coverage.rs      // Tests that exercise `unreferenced_layouts` / `layouts_without_scenarios` —
                            //   (the 4–6 layout-focused tests).
    invariant_kind_enumeration.rs // Tests that verify `InvariantKind::ALL` completeness —
                            //   (the tests that walk ALL variants or check NewVariant parity).
    formatting.rs           // `format_coverage_report` + `print_coverage_report` tests —
                            //   (~4 presentation tests).
  ```
- Writer-code: run `grep -nE '^fn|^#\[test\]' breaker-scenario-runner/src/coverage/tests.rs` to list all 28 test fns and place each into the group implied by its name + assertion target (missing_self_tests → self_test_coverage; unreferenced_layouts → layout_coverage; `InvariantKind::ALL` → invariant_kind_enumeration; `format_coverage_report` string assertions → formatting).
- Imports per sub-file: `use super::super::system::*;` plus `use super::helpers::minimal_scenario;` plus `use crate::types::{ChaosParams, InputStrategy, InvariantKind, ScenarioDefinition};` (same as original header).
- No change to `coverage/mod.rs` (still declares `#[cfg(test)] mod tests;`). No change to `coverage/system.rs`.
- Delegate: writer-code can execute directly.

---

## Recommended batching for parallel writer-code agents

Both HIGH items live in different crates and different subtrees — they can run in parallel.

- Group 1 (breaker-game crate): `echo_strike/tests/on_impact.rs` sub-split
- Group 2 (breaker-scenario-runner crate): `coverage/tests.rs` sub-split

After each wave, run Basic Verification Tier (`cargo all-dclippy` + `cargo all-dtest`).

## Hygiene

No stale split todos currently at the top of `docs/todos/TODO.md` — both prior split todos were correctly removed after completion. Add one new entry for this scan.

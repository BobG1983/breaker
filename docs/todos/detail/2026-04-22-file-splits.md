# File Splits — `rantzsoft_dmg/` (2026-04-22)

## Context

TODO #0 (`unified-death-crate.md`) shipped the `rantzsoft_dmg` crate across 7 phases.
Writer-tests deliberately co-located rich pipeline test suites with each P6 system file.
This sweep detects every `.rs` file in `rantzsoft_dmg/src/` and `rantzsoft_dmg/tests/`
that now exceeds the 400-line threshold in `.claude/rules/file-splitting.md`.

Scope: `rantzsoft_dmg/` only. No `breaker-game/` or other crate files touched.

## File Length Review

### Files Over Threshold

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| `rantzsoft_dmg/src/systems/handle_kill.rs` | 746 | 94 | 652 | 23 | A: test extraction | HIGH |
| `rantzsoft_dmg/src/plugin.rs` | 620 | 43 | 577 | 20 | A: test extraction | HIGH |
| `rantzsoft_dmg/src/app_ext.rs` | 583 | 69 | 514 | 17 | A: test extraction | HIGH |
| `rantzsoft_dmg/src/systems/apply_heal.rs` | 570 | 57 | 513 | 23 | A: test extraction | MEDIUM |
| `rantzsoft_dmg/src/systems/apply_damage_boosts.rs` | 510 | 29 | 481 | 17 | A: test extraction | MEDIUM |
| `rantzsoft_dmg/tests/cross_t_isolation.rs` | 464 | n/a | 464 | 10 | B: integration-test split | MEDIUM |
| `rantzsoft_dmg/src/systems/apply_vulnerable.rs` | 451 | 26 | 425 | 16 | A: test extraction | MEDIUM |
| `rantzsoft_dmg/tests/pipeline_integration/damage.rs` | 422 | n/a | 422 | 9 | LOW (monitor) | LOW |
| `rantzsoft_dmg/src/components/vulnerable_stack.rs` | 401 | 76 | 325 | 28 | A: test extraction | LOW |

### Borderline (under 400, monitor)

| File | Total | Notes |
|------|-------|-------|
| `rantzsoft_dmg/src/components/damage_boost_stack.rs` | 397 | Under threshold, monitor next sweep |
| `rantzsoft_dmg/src/systems/apply_damage.rs` | 397 | Under threshold, monitor next sweep |
| `rantzsoft_dmg/src/systems/detect_deaths.rs` | 336 | Safe |
| `rantzsoft_dmg/src/sets.rs` | 322 | Safe |
| `rantzsoft_dmg/src/lib.rs` | 319 | Safe |

### Priority Guide
- **HIGH**: 500+ lines — context-pollution pressure on any agent reading these files for editing or review
- **MEDIUM**: 401–500 lines — split to stay under threshold; low risk
- **LOW**: 400–450 test-only lines — flagged but no refactor spec; split if it grows

### mod.rs Violations

None. Every `mod.rs` in the crate is wiring-only (`src/systems/mod.rs`, `src/components/mod.rs`, `src/messages/mod.rs`, `src/traits/mod.rs`).

---

## Refactor Spec Hints

### 1. `rantzsoft_dmg/src/systems/handle_kill.rs` — HIGH

- Source file: `rantzsoft_dmg/src/systems/handle_kill.rs`
- Total lines: 746 (prod: 94 through line 94; tests: 652, lines 96–746)
- Strategy: **A — test extraction**
- Doc comment to preserve: the file-level `//! handle_kill::<T>` block (lines 1–13) moves to `system.rs`.
- Target structure:
  ```
  rantzsoft_dmg/src/systems/
    handle_kill/
      mod.rs      // pub(crate) use system::handle_kill; mod system; #[cfg(test)] mod tests;
      system.rs   // file-level doc + production (lines 1–94)
      tests.rs    // all tests (lines 96–746)
  ```
- Tests file size: 652 lines — below 800 threshold, keep as single `tests.rs`.
- Test groups (kept together in one `tests.rs`):
  - Insert-Dead / persistence (B111: `inserts_dead_on_victim`, `dead_persists_across_idle_tick`)
  - Destroyed + victim_pos (B112: `writes_destroyed_with_victim_pos`, `writes_destroyed_with_zero_victim_pos`)
  - killer_pos population (B113, B114, B115)
  - DespawnEntity emission (B116, B117)
  - Idempotency (B118, B119, B120, B120b regression, B121, B122)
  - Self-kill & HashSet reuse (B123, B124)
- Imports needed in `system.rs`: current imports from lines 15–24
- Imports needed in `tests.rs`: `use super::system::*;` plus the existing test-block imports (lines 98–107)
- Re-exports in `mod.rs`: `pub(crate) use system::handle_kill;` — matches `systems/mod.rs` line 18
- Parent change: **NONE** — `src/systems/mod.rs` line 9 (`mod handle_kill;`) resolves to either `handle_kill.rs` or `handle_kill/mod.rs`
- Delegate: writer-code can execute this refactor directly

### 2. `rantzsoft_dmg/src/plugin.rs` — HIGH

- Source file: `rantzsoft_dmg/src/plugin.rs`
- Total lines: 620 (prod: 43, tests: 577 starting at line 45)
- Strategy: **A — test extraction**
- Doc comment to preserve: the file-level `//!` block (lines 1–8) moves to `system.rs`.
- Target structure:
  ```
  rantzsoft_dmg/src/
    plugin/
      mod.rs      // pub use system::RantzDmgPlugin; mod system; #[cfg(test)] mod tests;
      system.rs   // file-level doc + plugin struct + impl (lines 1–43)
      tests.rs    // all tests (lines 45–620)
  ```
- Tests file size: 577 lines — below 800 threshold, keep as single `tests.rs`.
- Test groups (kept together in one `tests.rs`):
  - Derive and type contracts (B40–B44: `unit_struct_constructs_without_arguments`, `default_constructor_compiles`, `derives_debug_clone_copy_partial_eq_eq_hash`, `rantz_dmg_plugin_is_pub_at_crate_root`, `rantz_dmg_plugin_resolves_via_crate_glob_import`, `impl_plugin_and_app_builds_and_ticks`)
  - DespawnEntity registration (B45: `plugin_registers_despawn_entity_message`, `baseline_without_plugin_does_not_register_despawn_entity`)
  - Per-T isolation (B46: `plugin_does_not_register_per_t_messages`)
  - Set anchors and chain order (B47, B48, B49: `all_eleven_sets_are_usable_as_before_and_after_anchors`, `chain_order_matches_plan_after_one_tick`, `chain_order_stable_across_two_ticks`, `chain_is_in_fixed_update_not_update`)
  - process_despawn_requests scheduling (B50: `process_despawn_requests_scheduled_in_fixed_post_update`, `plugin_tolerates_empty_despawn_queue`, `plugin_tolerates_single_despawn_message_in_queue`)
  - Headless & non-injection (B52, B53)
- Imports in `tests.rs`: `use super::system::*;` plus the existing test-block imports (lines 47–54)
- Re-exports in `mod.rs`: `pub use system::RantzDmgPlugin;` — matches `lib.rs` line 47 (`pub use plugin::RantzDmgPlugin;`)
- Parent change: **NONE** — `src/lib.rs` line 36 (`mod plugin;`) resolves transparently
- Delegate: writer-code can execute this refactor directly

### 3. `rantzsoft_dmg/src/app_ext.rs` — HIGH

- Source file: `rantzsoft_dmg/src/app_ext.rs`
- Total lines: 583 (prod: 69 through line 69; tests: 514, lines 71–583)
- Strategy: **A — test extraction**
- Doc comment to preserve: the file-level `//!` block (lines 1–14) moves to `system.rs`.
- Target structure:
  ```
  rantzsoft_dmg/src/
    app_ext/
      mod.rs      // pub use system::RantzDmgAppExt; mod system; #[cfg(test)] mod tests;
      system.rs   // file-level doc + trait + impl (lines 1–69)
      tests.rs    // all tests (lines 71–583)
  ```
- Tests file size: 514 lines — below 800 threshold, keep as single `tests.rs`.
- Test groups (kept together in one `tests.rs`):
  - Message-resource registration (B139, B140)
  - Return type & chaining (B141, B142, B143)
  - Per-system wiring (B144, B145, B146, B147, B148)
  - T isolation & end-to-end (B149, B150, DespawnEntity witness)
- Imports in `tests.rs`: `use super::system::*;` plus the existing test-block imports (lines 73–83)
- Re-exports in `mod.rs`: `pub use system::RantzDmgAppExt;` — matches `lib.rs` line 42 (`pub use app_ext::RantzDmgAppExt;`)
- Parent change: **NONE** — `src/lib.rs` line 33 (`mod app_ext;`) resolves transparently
- Delegate: writer-code can execute this refactor directly

### 4. `rantzsoft_dmg/src/systems/apply_heal.rs` — MEDIUM

- Source file: `rantzsoft_dmg/src/systems/apply_heal.rs`
- Total lines: 570 (prod: 57 through line 57; tests: 513, lines 59–570)
- Strategy: **A — test extraction**
- Doc comment to preserve: the file-level `//!` block (lines 1–13) moves to `system.rs`.
- Target structure:
  ```
  rantzsoft_dmg/src/systems/
    apply_heal/
      mod.rs      // pub(crate) use system::apply_heal; mod system; #[cfg(test)] mod tests;
      system.rs   // file-level doc + production (lines 1–57)
      tests.rs    // all tests (lines 59–570)
  ```
- Tests file size: 513 lines — below 800 threshold, keep as single `tests.rs`.
- Test groups (kept together in one `tests.rs`):
  - Increment below ceiling (B125, B126)
  - HealCap semantics (B127, B128, B129)
  - Dead / Invulnerable gating (B130, B131)
  - Over-cap short-circuit (B132)
  - Amount guards (B133: zero/negative/nan/inf)
  - T marker filter (B134)
- Imports in `tests.rs`: `use super::system::*;` plus the existing test-block imports (lines 61–69)
- Re-exports in `mod.rs`: `pub(crate) use system::apply_heal;` — matches `systems/mod.rs` line 14
- Parent change: **NONE** — `src/systems/mod.rs` line 5 (`mod apply_heal;`) resolves transparently
- Delegate: writer-code can execute this refactor directly

### 5. `rantzsoft_dmg/src/systems/apply_damage_boosts.rs` — MEDIUM

- Source file: `rantzsoft_dmg/src/systems/apply_damage_boosts.rs`
- Total lines: 510 (prod: 29, tests: 481 starting at line 31)
- Strategy: **A — test extraction**
- Doc comment to preserve: the file-level `//!` block (lines 1–7) moves to `system.rs`.
- Target structure:
  ```
  rantzsoft_dmg/src/systems/
    apply_damage_boosts/
      mod.rs      // pub(crate) use system::apply_damage_boosts; mod system; #[cfg(test)] mod tests;
      system.rs   // file-level doc + production (lines 1–29)
      tests.rs    // all tests (lines 31–510)
  ```
- Tests file size: 481 lines — below 800 threshold, keep as single `tests.rs`.
- Test groups (kept together in one `tests.rs`):
  - Persistent-only (B82)
  - One-shot-only (B83)
  - Both lanes (B84)
  - Multi-entry persistent (B85)
  - Pass-through & isolation (B86)
- Imports in `tests.rs`: `use super::system::*;` plus the existing test-block imports (lines 33–38)
- Re-exports in `mod.rs`: `pub(crate) use system::apply_damage_boosts;` — matches `systems/mod.rs` line 13
- Parent change: **NONE** — `src/systems/mod.rs` line 4 resolves transparently
- Delegate: writer-code can execute this refactor directly

### 6. `rantzsoft_dmg/tests/cross_t_isolation.rs` — MEDIUM

- Source file: `rantzsoft_dmg/tests/cross_t_isolation.rs`
- Total lines: 464 (integration-test binary; entire file is tests)
- Strategy: **B — integration-test split** (same idiom as `tests/pipeline_integration/`)
- Target structure:
  ```
  rantzsoft_dmg/tests/
    cross_t_isolation.rs          // top-level binary: #![cfg_attr(...)] + #[path] mod declarations
    cross_t_isolation/
      damage.rs                   // damage cross-T tests (B11–12)
      heal.rs                     // heal cross-T tests (B13)
      kill.rs                     // kill+destroy cross-T tests (B14)
      dealer_boost.rs             // cross-T boost-on-single-dealer tests
      invulnerable.rs             // cross-T invulnerable isolation
  ```
- Test groups (by behavior, see line numbers in source):
  - `damage.rs`: `damage_dealt_t1_does_not_affect_t2_target` (l.69), `damage_dealt_t2_affects_t2_target_and_leaves_t1_alone` (l.106) — ~70 lines
  - `heal.rs`: `heal_dealt_t1_does_not_affect_t2_target` (l.131), `heal_dealt_t2_affects_t2_target` (l.174) — ~80 lines
  - `kill.rs`: `dual_t_kill_emits_one_destroyed_per_type_with_correct_killer` (l.209), `single_t1_kill_leaves_t2_victim_alive_and_t2_destroyed_queue_empty` (l.270) — ~110 lines
  - `dealer_boost.rs`: `persistent_boost_on_single_dealer_multiplies_both_t_queues` (l.316), `one_shot_on_single_dealer_drained_by_whichever_t_system_runs_first` (l.353) — ~80 lines
  - `invulnerable.rs`: `invulnerable_on_t1_does_not_shield_t2_victim` (l.398), `invulnerable_on_t2_does_not_shield_t1_victim` (l.432) — ~70 lines
- Shared helpers: `assert_f32_eq`, `T1`, `T2`, `app_with_both_types`, `tick` — duplicate in each sub-file (per existing crate test-copy-paste policy observed in `apply_*` files) OR place in a `cross_t_isolation/common.rs` with `pub(super)` items. **Recommended: dedicate `cross_t_isolation/common.rs` with `pub(super) use` of shared helpers** because the five sub-files all use identical setup.
- Top-level binary contents after split:
  ```rust
  //! P7 crate integration tests — cross-`T` isolation (behaviors 11–15).
  #![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, reason = "integration tests use unwrap/expect/panic for assertion clarity"))]

  #[path = "cross_t_isolation/common.rs"]
  mod common;
  #[path = "cross_t_isolation/damage.rs"]
  mod damage;
  #[path = "cross_t_isolation/heal.rs"]
  mod heal;
  #[path = "cross_t_isolation/kill.rs"]
  mod kill;
  #[path = "cross_t_isolation/dealer_boost.rs"]
  mod dealer_boost;
  #[path = "cross_t_isolation/invulnerable.rs"]
  mod invulnerable;
  ```
  (Matches the exact `#[path]` idiom used in `tests/pipeline_integration.rs`.)
- Imports per sub-file: current imports lines 18–25 of the original plus `use super::common::*;`
- Parent change: **NONE** in `Cargo.toml` — `tests/cross_t_isolation.rs` is still the binary root; Cargo auto-discovers `tests/*.rs` at the top level
- Delegate: writer-code can execute this refactor directly

### 7. `rantzsoft_dmg/src/systems/apply_vulnerable.rs` — MEDIUM

- Source file: `rantzsoft_dmg/src/systems/apply_vulnerable.rs`
- Total lines: 451 (prod: 26, tests: 425 starting at line 28)
- Strategy: **A — test extraction**
- Doc comment to preserve: the file-level `//!` block (lines 1–7) moves to `system.rs`.
- Target structure:
  ```
  rantzsoft_dmg/src/systems/
    apply_vulnerable/
      mod.rs      // pub(crate) use system::apply_vulnerable; mod system; #[cfg(test)] mod tests;
      system.rs   // file-level doc + production (lines 1–26)
      tests.rs    // all tests (lines 28–451)
  ```
- Tests file size: 425 lines — below 800 threshold, keep as single `tests.rs`.
- Test groups (kept together in one `tests.rs`):
  - Persistent-only (B87)
  - One-shot-only (B88)
  - Both lanes (B89)
  - Pass-through & empty (B90)
  - Dealer-boost-ignored target-keyed (B91)
- Imports in `tests.rs`: `use super::system::*;` plus the existing test-block imports (lines 30–39)
- Re-exports in `mod.rs`: `pub(crate) use system::apply_vulnerable;` — matches `systems/mod.rs` line 15
- Parent change: **NONE** — `src/systems/mod.rs` line 6 resolves transparently
- Delegate: writer-code can execute this refactor directly

### 8. `rantzsoft_dmg/src/components/vulnerable_stack.rs` — LOW (flagged)

- Source file: `rantzsoft_dmg/src/components/vulnerable_stack.rs`
- Total lines: 401 (prod: 76 through line 76; tests: 325, lines 77–401)
- Strategy: **A — test extraction**
- Target structure:
  ```
  rantzsoft_dmg/src/components/
    vulnerable_stack/
      mod.rs      // pub(crate) use system::VulnerableStack; mod system; #[cfg(test)] mod tests;
      system.rs   // file-level doc + VulnerableStack (lines 1–76)
      tests.rs    // all tests (lines 77–401)
  ```
- Parent change: **NONE** — `src/components/mod.rs` resolves transparently
- Re-exports in `mod.rs`: `pub(crate) use system::VulnerableStack;` (or whatever visibility matches the current `pub`)
- Only 1 line over threshold — defer if time is short, but the refactor is mechanical and risk-free
- Delegate: writer-code can execute this refactor directly

---

## Batching for Parallel Writer-Code

All files live in the same crate (`rantzsoft_dmg`). Split agents run them in parallel — each
converts a single `.rs` file to a directory module and writes `mod.rs` LAST per the split procedure
in `.claude/rules/file-splitting.md`. No cross-file dependencies; no parent-module edits required.

- **Wave A (systems/)**: `handle_kill`, `apply_heal`, `apply_damage_boosts`, `apply_vulnerable` — 4 writer-code agents in parallel; all touch `src/systems/` only.
- **Wave B (crate root)**: `plugin.rs`, `app_ext.rs` — 2 writer-code agents in parallel; both touch `src/`.
- **Wave C (integration test)**: `tests/cross_t_isolation.rs` — 1 writer-code agent; touches `tests/` only.
- **Wave D (low priority, defer if needed)**: `components/vulnerable_stack.rs` — 1 writer-code agent.

All four waves are independent and could launch simultaneously (different directory subtrees).

## Post-Split Verification

After each split batch, the orchestrator must:
1. Remove the orphaned `*.rs` files the writer-code leaves behind (Rust resolves `foo/mod.rs` once
   created, but both files co-exist — orchestrator cleans up `foo.rs`).
2. Run Basic Verification Tier (`runner-linting` + `runner-tests`) per the file-splitting routing
   rule in `.claude/rules/routing-failures.md`.
3. Use `cargo all-dtest` and `cargo all-dclippy` (see `.claude/rules/cargo.md`) — these cover
   `rantzsoft_dmg` as a workspace member.

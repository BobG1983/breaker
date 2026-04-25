# File splits — mutators domain (refactor/consolidate-mutators-domain)

Audit of the `mutators/` consolidation (was `hazard/` + `protocol/`). Threshold = 400 lines. Strategies follow `.claude/rules/file-splitting.md`.

## Summary

**Update (2026-04-25):** The HIGH item below was addressed on `refactor/consolidate-mutators-domain` — `mutators/hazards/mod.rs` was split into `fan_out.rs` + `tests.rs`, and the sibling `mutators/protocols/mod.rs` was split the same way for consistency. The remaining MEDIUM/LOW items are pre-existing borderline files unrelated to the consolidation refactor and remain in scope for this todo.

| File | Total | Prod | Tests | Test Fns | Strategy | Priority |
|------|-------|------|-------|----------|----------|----------|
| ~~`breaker-game/src/mutators/hazards/mod.rs`~~ | ~~168~~ | ~~102~~ | ~~66~~ | ~~2~~ | ~~mod.rs violation~~ | DONE |
| `breaker-game/src/mutators/protocols/iron_curtain/tests/on_bolt_lost.rs` | 710 | 0 | 710 | ~18 | C: split test groups | MEDIUM |
| `breaker-game/src/mutators/protocols/definition.rs` | 642 | 264 | 378 | ~10 | A: extract tests | MEDIUM |
| `breaker-game/src/mutators/hazards/tether/tests/establish_links.rs` | 598 | 0 | 598 | ~12 | C: split test groups | MEDIUM |
| `breaker-game/src/mutators/hazards/definition.rs` | 566 | ~? | ~? | ~? | A: extract tests | MEDIUM |
| `breaker-game/src/mutators/protocols/burnout/tests/update_heat.rs` | 540 | 0 | 540 | ~12 | C: split test groups | MEDIUM |
| `breaker-game/src/mutators/protocols/resources.rs` | 476 | 205 | 271 | ~12 | A: extract tests | LOW |
| `breaker-game/src/mutators/hazards/resources.rs` | 404 | ~? | ~? | ~? | A: extract tests | LOW |
| `breaker-game/src/mutators/hazards/diffusion/tests/emit_rings.rs` | 409 | 0 | 409 | ~9 | C: monitor | LOW |

Files explicitly checked and **CLEAN** (under 400):
- `mutators/protocols/echo_strike/system.rs` — 343
- `mutators/hazards/diffusion/system.rs` — 357
- `mutators/hazards/tether/system.rs` — 339
- `mutators/plugin/system.rs` — 147
- `mutators/protocols/echo_strike/tests/emit_siblings.rs` — 348
- `mutators/protocols/burnout/tests/on_bump.rs` — 362
- `mutators/protocols/burnout/tests/amplify.rs` — 404 (LOW border, not flagged)
- `mutators/protocols/debt_collector/tests/on_impact.rs` — 362
- `mutators/protocols/reckless_dash/tests/double_penalty.rs` — 306
- `mutators/protocols/reckless_dash/system.rs` — 341

## Priority Guide
- **HIGH**: mod.rs violations or 1000+ total / 800+ test lines
- **MEDIUM**: 501-999 lines (split at least once)
- **LOW**: 400-500 lines (flag, will need splitting soon)

---

## HIGH — `mutators/hazards/mod.rs` (mod.rs violation)

**Refactor spec hint:**
- Source file: `breaker-game/src/mutators/hazards/mod.rs`
- Total lines: 168 (prod fan-out: 102, inline tests: 66 — `#[cfg(test)] mod tests`)
- Strategy: B-style — `mod.rs` must contain ONLY module declarations and the `pub use` of `MutatorsPlugin`-adjacent items. Production fan-out (`activate`, `wire`, `activate_from_registry`) and the inline tests both need to leave `mod.rs`.
- Target structure:
  ```
  breaker-game/src/mutators/hazards/
    mod.rs              // module declarations + re-exports ONLY
    fan_out.rs          // pub(crate) fn activate / wire / activate_from_registry
    tests.rs            // the existing #[cfg(test)] mod tests block
  ```
- New `mod.rs` contents (only):
  ```rust
  //! Hazard domain — negative stackable hazards chosen during infinite play
  //! at tier 9 and above. […keep doc comment…]

  pub mod definition;
  pub(crate) mod messages;
  pub mod resources;
  pub(crate) mod systems;

  pub(crate) mod cascade;
  pub(crate) mod decay;
  pub(crate) mod diffusion;
  pub(crate) mod drift;
  pub(crate) mod echo_cells;
  pub(crate) mod erosion;
  pub(crate) mod fracture;
  pub(crate) mod gravity_surge;
  pub(crate) mod haste;
  pub(crate) mod momentum;
  pub(crate) mod overcharge;
  pub(crate) mod renewal;
  pub(crate) mod resonance;
  pub(crate) mod sympathy;
  pub(crate) mod tether;
  pub(crate) mod volatility;

  pub(crate) mod fan_out;
  pub(crate) use fan_out::{activate, wire, activate_from_registry};

  #[cfg(test)]
  mod tests;
  ```
- `fan_out.rs`: lines 31–103 of current `mod.rs` (the `use` block, `activate`, `wire`, `activate_from_registry`).
- `tests.rs`: lines 105–168 of current `mod.rs` (the inline `#[cfg(test)] mod tests` body, but as a top-level module — drop the wrapping `mod tests {}`).
- Re-exports needed: `activate_from_registry` is the public escape hatch (used by the scenario runner — keep it `pub`); `activate` and `wire` are `pub(crate)` and used by `mutators/plugin/system.rs`.
- Imports needed in `fan_out.rs`: `bevy::prelude::*`, `super::{definition::{HazardKind, HazardTuning}, resources::HazardRegistry}`, `super::{cascade, decay, diffusion, …}`.
- Imports needed in `tests.rs`: `super::{decay::DecayConfig, definition::*, resources::HazardRegistry, *}` and `crate::prelude::TestAppBuilder`. Note: the inline tests refer to `super::*` (the fan_out symbols) so move `pub(crate) use fan_out::activate_from_registry` to `mod.rs` as already shown — `tests.rs` accesses it through `super::activate_from_registry` (resolved via `mod.rs`'s `pub(crate) use`).
- Delegate: writer-code can execute directly via `/quickfix`.

---

## MEDIUM — `mutators/protocols/iron_curtain/tests/on_bolt_lost.rs`

**Refactor spec hint:**
- Source file: `breaker-game/src/mutators/protocols/iron_curtain/tests/on_bolt_lost.rs`
- Total lines: 710 (all tests)
- Strategy: C — convert `on_bolt_lost.rs` into a test directory `on_bolt_lost/`. The parent `tests/mod.rs` `mod on_bolt_lost;` line stays unchanged.
- Target structure:
  ```
  breaker-game/src/mutators/protocols/iron_curtain/tests/
    on_bolt_lost/
      mod.rs            // mod helpers; mod activation_gate; mod falloff_math; mod source_attribution; mod edge_cases;
      helpers.rs        // any inline test helpers in the current file
      activation_gate.rs    // tests gated on ActiveProtocols / NodeState (early-out behaviors)
      falloff_math.rs       // tests around damage_fraction × falloff_start formula (B7-B14 range)
      source_attribution.rs // B30 source builder test + dealer/attributed_to invariants
      edge_cases.rs         // missing-breaker, leaked-message-prevention, degenerate-max-distance (B15-B18)
  ```
- Test groups (read the file's banner comments — they already number behaviors):
  - `activation_gate.rs`: tests where `ActiveProtocols`/`IronCurtainConfig`/`NodeState` is missing or off
  - `falloff_math.rs`: distance-based amount calculations
  - `source_attribution.rs`: `SourceId::protocol(IronCurtain).build()` matching, dealer/attributed_to wiring
  - `edge_cases.rs`: empty breaker query, second-tick leak prevention (B16-B17), degenerate falloff (B18)
- Imports needed (each group): mirror the current file header — `use super::super::helpers::*` plus the iron_curtain config/system imports
- Delegate: writer-code executes after re-reading the file to confirm test grouping.

---

## MEDIUM — `mutators/protocols/definition.rs`

**Refactor spec hint:**
- Source file: `breaker-game/src/mutators/protocols/definition.rs`
- Total lines: 642 (prod: 264, tests: ~378)
- Strategy: A — convert to a directory module so the inline `#[cfg(test)] mod tests` exits to its own file.
- Target structure:
  ```
  breaker-game/src/mutators/protocols/definition/
    mod.rs        // pub(crate) mod system; pub use system::*; #[cfg(test)] mod tests;
    system.rs     // current lines 1-264 (production code)
    tests.rs      // current lines 265-642 (the existing test module body, hoisted)
  ```
- Re-exports needed (from current public surface): `ProtocolKind`, `ProtocolDefinition`, `ProtocolTuning` and any `pub`/`pub(crate)` items used by `mutators/protocols/mod.rs` and `mutators/protocols/resources.rs`.
- Imports needed in `tests.rs`: `use super::system::*;` plus `use std::collections::HashSet;`.
- Delegate: writer-code via `/quickfix`.

---

## MEDIUM — `mutators/hazards/tether/tests/establish_links.rs`

**Refactor spec hint:**
- Source file: `breaker-game/src/mutators/hazards/tether/tests/establish_links.rs`
- Total lines: 598 (all tests)
- Strategy: C — convert to `establish_links/` directory. Parent `tests/mod.rs` line `mod establish_links;` is unchanged.
- Target structure:
  ```
  breaker-game/src/mutators/hazards/tether/tests/establish_links/
    mod.rs                 // mod adjacency; mod coverage_math; mod determinism; mod edge_cases;
    adjacency.rs           // pair-eligibility + ADJACENCY_RADIUS_SQ tests
    coverage_math.rs       // base_coverage / coverage_per_level percent math
    determinism.rs         // GameRng-driven deterministic selection tests
    edge_cases.rs          // empty pool, single-cell, mutual-exclusion conflicts
  ```
- Group split is by **behavior** (not test name). Writer-code reads the file and groups based on banner comments / `// ── Behavior N ──` markers.
- Delegate: writer-code via `/quickfix`.

---

## MEDIUM — `mutators/hazards/definition.rs`

**Refactor spec hint:**
- Source file: `breaker-game/src/mutators/hazards/definition.rs`
- Total lines: 566
- Strategy: A — extract tests. Same pattern as `protocols/definition.rs`.
- Target structure:
  ```
  breaker-game/src/mutators/hazards/definition/
    mod.rs        // pub(crate) mod system; pub use system::*; #[cfg(test)] mod tests;
    system.rs     // production
    tests.rs      // hoisted #[cfg(test)] block
  ```
- Re-exports needed: `HazardKind`, `HazardDefinition`, `HazardTuning` (used widely across the hazards subtree and by `mutators/hazards/mod.rs`).
- Delegate: writer-code via `/quickfix`.

---

## MEDIUM — `mutators/protocols/burnout/tests/update_heat.rs`

**Refactor spec hint:**
- Source file: `breaker-game/src/mutators/protocols/burnout/tests/update_heat.rs`
- Total lines: 540 (all tests)
- Strategy: C — convert to `update_heat/` directory.
- Target structure:
  ```
  breaker-game/src/mutators/protocols/burnout/tests/update_heat/
    mod.rs               // mod fill_drain; mod still_threshold; mod gates; mod edge_cases;
    fill_drain.rs        // fill_duration / drain_duration math
    still_threshold.rs   // still-detection gating
    gates.rs             // ActiveProtocols / NodeState run-condition tests
    edge_cases.rs        // 0 heat, max heat, mid-tick state changes
  ```
- Delegate: writer-code via `/quickfix` after reading the file to confirm groupings.

---

## LOW (flag, do not split this round)

- `mutators/protocols/resources.rs` (476) — Strategy A candidate but test ratio is 57%; split next time it crosses 500 or test ratio crosses 60%.
- `mutators/hazards/resources.rs` (404) — at threshold; same trigger.
- `mutators/hazards/diffusion/tests/emit_rings.rs` (409) — borderline, monitor.

---

## Recommended batching for parallel writer-code

All splits are within `breaker-game/src/mutators/`. Each leaf test directory is independent of the others (they don't share files). Suggested two parallel batches:

**Batch A** — production-side / mod.rs:
1. `mutators/hazards/mod.rs` (HIGH, mod.rs violation)
2. `mutators/protocols/definition.rs` (MEDIUM, A)
3. `mutators/hazards/definition.rs` (MEDIUM, A)

**Batch B** — test directory conversions (Strategy C, fully parallel-safe):
1. `mutators/protocols/iron_curtain/tests/on_bolt_lost.rs`
2. `mutators/hazards/tether/tests/establish_links.rs`
3. `mutators/protocols/burnout/tests/update_heat.rs`

Run Basic Verification Tier (`runner-linting` + `runner-tests`) after each batch.

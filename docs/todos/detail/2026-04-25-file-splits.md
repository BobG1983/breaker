# File Splits — port branch W7/W8 sweep (2026-04-25)

## Context

Full-tier sweep on `feature/port-breaker-to-rantzsoft-dmg` post-W7/W8
(commits `f4e54383` `EmitHeal`, `976dc9ce` `DamageBoost/Vulnerable`,
`be5ebf40` initial port, `e612c4c9` Phase 19/20 splits, `a33a2126`).

Phase 20 actionable HIGH/MEDIUM items are all CONFIRMED resolved:
- `apply_damage.rs` — directory module exists (no offending file at original path)
- `lost_detection_tests.rs` — file removed; replaced by sub-split directory
- `lib.rs` — 132 lines (was 408; tests extracted to sibling)

## File Length Review

### Files Over Threshold

| File | Total | Prod | Tests | Strategy | Priority |
|------|-------|------|-------|----------|----------|
| `breaker-game/src/protocol/protocols/burnout/system.rs` | 440 | 440 | 0 | B: concern separation | MEDIUM |
| `rantzsoft_dmg/src/plugin/tests.rs` | 718 | 0 | 718 | C (monitor) | MEDIUM |
| `rantzsoft_dmg/src/systems/handle_kill/tests.rs` | 650 | 0 | 650 | C (monitor) | LOW |
| `rantzsoft_dmg/src/systems/apply_damage_boosts/tests.rs` | 566 | 0 | 566 | C (monitor) | LOW |
| `rantzsoft_dmg/src/systems/apply_heal/tests.rs` | 512 | 0 | 512 | C (monitor) | LOW |
| `rantzsoft_dmg/src/app_ext/tests.rs` | 521 | 0 | 521 | C (monitor) | LOW |
| `rantzsoft_dmg/tests/pipeline_integration/damage.rs` | 436 | 0 | 436 | C (monitor) | LOW |
| `breaker-game/src/bolt/systems/bolt_cell_collision/tests/damage_messages.rs` | 432 | 0 | 432 | C (defer to TODO #2) | LOW |
| `breaker-game/src/protocol/protocols/echo_strike/system.rs` | 351 | 351 | 0 | — | (under) |
| `breaker-game/src/protocol/protocols/iron_curtain/system.rs` | 195 | 195 | 0 | — | (under) |
| `breaker-game/src/protocol/protocols/reckless_dash/system.rs` | 342 | 342 | 0 | — | (under) |
| `breaker-game/src/protocol/protocols/debt_collector/system.rs` | 302 | 302 | 0 | — | (under) |
| `breaker-game/src/hazard/hazards/diffusion/system.rs` | 372 | 372 | 0 | — | (under, +1 sweep) |
| `breaker-game/src/hazard/hazards/tether/system.rs` | 340 | 340 | 0 | — | (under) |

### Priority Guide
- **HIGH**: 700+ lines or 800+ test-only — none this sweep
- **MEDIUM**: 401–500 production, or 700+ test-only one sub-split away
- **LOW**: borderline / monitor — all of these were flagged in 04-23 and remain stable

### mod.rs Violations
None — every probed `tests/mod.rs` contains only `mod` declarations.

---

## Refactor Spec Hints

### 1. `breaker-game/src/protocol/protocols/burnout/system.rs` — MEDIUM (only actionable item)

- Source: `breaker-game/src/protocol/protocols/burnout/system.rs`
- Total: 440 (all production; tests already in `tests/` directory)
- Strategy: **B — concern separation by sub-system**
- Rationale: file declares 5 distinct systems plus the `BurnoutConfig` resource and a cleanup helper. Lines split cleanly along the `// ── System N — name ───` banners.
- Target structure:
  ```
  protocol/protocols/burnout/
    mod.rs                    // UNCHANGED
    system/
      mod.rs                  // pub(crate) use: re-exports each system fn + BurnoutConfig
      config.rs               // BurnoutConfig + ProtocolTuning impl
      update_heat.rs          // System 1
      on_bump.rs              // System 2 (heat-driven shockwave + damage boost)
      amplify.rs              // System 3 (heat-amp damage curve)
      tick_speed_boost.rs     // System 4 (BurnoutSpeedBoost decay)
      cleanup_node.rs         // System 5 (OnExit cleanup)
  ```
- Imports: each sub-file copies the cross-cutting block (lines 5–22) trimmed to what each uses.
- Re-exports in `system/mod.rs`: every `pub(crate) fn` previously in the flat file, plus `BurnoutConfig`.
- Parent (`burnout/mod.rs`) — UNCHANGED. The `pub(crate) mod system;` line resolves to the new directory.
- Test impact: tests import via `use super::system::*;` (or the `super::super::system::*;` two-up form) — re-exports keep all names visible, so no test edits required.
- Delegate: writer-code can execute directly.

### 2. `rantzsoft_dmg/src/plugin/tests.rs` — MEDIUM (monitor; do not split yet)

- Total: 718 (unchanged from 04-23). 82 lines below sub-split trigger.
- Action: **none this sweep**. Pre-define the sub-split groups now so a future split is a mechanical move:
  - `derive_contracts.rs` (B40–B44)
  - `despawn_registration.rs` (B45)
  - `per_t_isolation.rs` (B46)
  - `chain_ordering.rs` (B47–B49)
  - `fixed_post_update.rs` (B50)
  - `headless_noninjection.rs` (B52, B53)

### 3–8. LOW monitor

| File | Total | Status |
|------|-------|--------|
| `handle_kill/tests.rs` | 650 | stable |
| `apply_damage_boosts/tests.rs` | 566 | stable (was 566 on 04-23) |
| `apply_heal/tests.rs` | 512 | stable |
| `app_ext/tests.rs` | 521 | stable |
| `pipeline_integration/damage.rs` | 436 | stable |
| `bolt_cell_collision/tests/damage_messages.rs` | 432 | defer until TODO #2 (mutators consolidation) lands to avoid churn |

---

## Batching for Parallel Writer-Code

- **Wave A**: 1 agent for `burnout/system.rs` Strategy B split (single domain, no cross-crate dependencies). Pure file moves + import trims; no logic change.

## Post-Split Verification

After the split:
1. Orchestrator removes orphaned `burnout/system.rs` once `burnout/system/` directory exists.
2. Run Basic Verification Tier (`cargo all-dtest` + `cargo all-dclippy`).
3. Confirm no test imports broke — `system/mod.rs` re-exports must mirror the original flat file's `pub(crate)` surface.

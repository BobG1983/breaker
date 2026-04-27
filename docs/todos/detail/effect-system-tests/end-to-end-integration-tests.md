# Effect system: end-to-end integration tests across every primitive

## Problems addressed

- `audit/protocols/kickstart.md` Issue 2 / 6 / 7 \u2014 `Until(TimeExpires)` shipped unwired; no test caught it because every component had unit tests but no test wired the full RON \u2192 scope-entry \u2192 tick \u2192 expire \u2192 reverse pipeline end-to-end. This remediation generalizes the lesson to EVERY effect-system primitive to find and prevent similar latent gaps.

## Why

Unit tests of individual pieces (the walker, the tick system, the bridge) pass while the integration stays broken. The `Until(TimeExpires)` bug is the archetype: every piece tested in isolation, no piece testing the seam between them. commit `945254fd` (`fix(effect_v3): wire Until(TimeExpires) end-to-end`) fixes the specific gap; this remediation systematically prevents the category.

## Remediation

### Coverage matrix

Audit every primitive the effect system supports. For each, write (or verify) an end-to-end test that exercises the FULL pipeline with no fixture-spawn shortcuts:

| Primitive | Scope-entry action | Tick/fire path | Scope-exit action | E2E test goes in |
|-----------|--------------------|----------------|--------------------|------------------|
| `When(trigger, inner)` | Inner fires when trigger dispatches | Trigger bridge \u2192 walker \u2192 inner fire | N/A (one-shot) | `effect_v3/triggers/<trigger>/tests/` |
| `Until(trigger, inner)` | Inner fires on entry, timer/condition installed | Trigger bridge \u2192 reverse inner | Reverse fires | `effect_v3/conditions/evaluate_conditions/tests/` |
| `During(condition, inner)` | Inner fires when condition becomes true, reverses when false | condition evaluator \u2192 fire/reverse | N/A (continuous) | `effect_v3/conditions/evaluate_conditions/tests/` |
| `On(participant, inner)` | Inner fires on participant | participant evaluator | N/A | `effect_v3/conditions/evaluate_conditions/tests/` |
| `Stamp(target, tree)` | Tree stamped on target set | dispatch bridge | N/A | `effect_v3/dispatch/tests/` |
| `Fire(effect)` | Effect appended to EffectStack | aggregation consumer | N/A | `effect_v3/dispatch/tests/` |
| `Route(effect)` | Effect routed to target | route resolver | N/A | `effect_v3/dispatch/tests/` |

For each row, the E2E test:

1. Builds a headless app with the effect-system plugin + any required domain plugins (bolt, cell, etc. depending on what triggers/effects the test exercises).
2. Spawns an entity with a `BoundEffects` tree containing the primitive under test.
3. Drives the real-world trigger that SHOULD activate the scope (e.g., `BoltImpactWall` message for `Impacted(Wall)`, `OnEnter(NodeState::Playing)` for `NodeStartOccurred`, a frame tick for `TimeExpires`, etc.).
4. Asserts the observable outcome: effect landed in EffectStack, component inserted, message dispatched, scope reversed.

Crucially, the test does NOT manually spawn `EffectTimers`, `EffectStack` entries, or intermediate state. The full pipeline must run from real triggers. Fixture-spawn tests stay in place for unit coverage, but they do NOT substitute for E2E coverage.

### Known gaps to fill FIRST

Based on the audit, start with the primitives most likely to have the `Until(TimeExpires)` pattern:

1. **`Until(TimeExpires(duration), ...)`** \u2014 covered by commit `945254fd` (`fix(effect_v3): wire Until(TimeExpires) end-to-end`) (fix + E2E test).
2. **`Until(Impacted(EntityKind), ...)`** \u2014 Ricochet's canonical pattern. Verify an E2E test exists that walks `Impacted(Wall)` all the way through to a DamageBoost on the bolt's EffectStack. If not, write one.
3. **`When(NodeStartOccurred, ...)`** \u2014 Kickstart's trigger. Verify E2E: `OnEnter(NodeState::Playing)` \u2192 bridge fires NodeStartOccurred \u2192 walker fires inner.
4. **`Stamp(EveryBolt, ...)`** \u2014 the audit's Issue 1 cross-cutting bug. Once `stamp-dispatcher-unification.md` lands, add an E2E test that declares `Stamp(EveryBolt, ...)` in a test-only RON fixture and asserts the tree lands on all bolts (not the breaker).
5. **`Stamp(EveryCell, ...)`** / `ActiveCells` / `ActiveBolts` / etc. \u2014 every `StampTarget` variant gets an E2E test.
6. **`Route(...)`** \u2014 same E2E treatment for every route target.

### Test file organization

Group E2E tests under `breaker-game/src/effect_v3/e2e/` (new module) with one file per primitive:

```
effect_v3/e2e/
  mod.rs
  when_trigger.rs
  until_time_expires.rs
  until_impacted.rs
  during_condition.rs
  on_participant.rs
  stamp_every_bolt.rs
  stamp_every_cell.rs
  stamp_active_bolts.rs
  stamp_active_cells.rs
  route_<target>.rs
  fire_<effect>.rs        // one per EffectType variant
```

Each file has a handful of tests covering the primitive's basic shape, edge cases (scope canceled, target missing, timer expiring mid-frame, etc.), and integration with at least one other primitive (e.g., `Until(TimeExpires)` nested inside `During(...)`).

### Naming convention

Test function names follow `<primitive>_<scenario>_<expected_outcome>`:

```rust
#[test]
fn until_time_expires_installs_timer_on_scope_entry() { ... }

#[test]
fn until_time_expires_reverses_inner_on_timer_expiration() { ... }

#[test]
fn until_time_expires_cancels_timer_on_outer_reversal() { ... }

#[test]
fn stamp_every_bolt_lands_tree_on_all_bolts_not_breaker() { ... }
```

### Acceptance criteria

This remediation is complete when:

1. Every primitive in `effect_v3/types/` (every `Trigger`, `Condition`, `Participant`, `StampTarget`, `RouteTarget`, `EffectType`, `Tree` variant) has at least one E2E test in `effect_v3/e2e/`.
2. Every E2E test drives the full pipeline from a "real" trigger (no fixture-spawned `EffectTimers`, `EffectStack`, armed-scope entries, etc.).
3. A coverage check (manual or scripted) confirms every variant is touched.
4. Any gap discovered during the sweep (a primitive that silently no-ops because a middle step is unwired) is filed as its own remediation or folded into commit `945254fd` (`fix(effect_v3): wire Until(TimeExpires) end-to-end`) if it's an analogous wiring bug.

### Effort sizing

This is a multi-session effort. Recommend splitting into phases matching the primitive table:

- Phase 1: `Until(TimeExpires)` \u2014 covered by commit `945254fd` (`fix(effect_v3): wire Until(TimeExpires) end-to-end`).
- Phase 2: Other `Until(trigger, ...)` variants (Impacted, Bump, etc.).
- Phase 3: `During(condition, ...)` + `On(participant, ...)`.
- Phase 4: `Stamp(target, ...)` every variant.
- Phase 5: `Route(...)` + `Fire(...)` per EffectType.

Each phase lands as its own commit with its own test files. Phase ordering is flexible; prioritize primitives most used by protocols/hazards.

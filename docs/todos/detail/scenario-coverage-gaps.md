# Scenario Coverage Gaps

## Summary
Fill scenario coverage gaps for the chip evolution lifecycle and untested trigger variants (NoBump, Died, Impact(Bolt), Impacted(Breaker)).

## Context
The scenario runner has 100+ scenarios covering bumps, dash states, chip offerings, entity leaks, speed, and various mechanic-specific behaviors. But some trigger variants in the `Trigger` enum have no scenario exercising them, and the full chip evolution lifecycle (acquire → stack → evolve → verify evolved effects) has no end-to-end scenario.

This is a coverage completeness concern, not a suspected-bugs concern. The trigger systems have unit tests, but no adversarial scenario exercises them under chaos input.

## Scope

### In
- **Bolt birthing collision suppression scenario**: verify birthing bolts cannot collide (zeroed CollisionLayers). New `BoltBirthingLayersZeroed` invariant needed.
- **Bolt birthing respawn scenario**: bolt lost → respawn inserts Birthing, collision suppressed for 0.3s. Chaos input, 3000 frames.
- **Effect-spawned bolt birthing**: add `BoltBirthingLayersZeroed` invariant to existing effect scenarios (mirror_protocol_chaos, circuit_breaker_chaos, nova_lance_chaos, split_decision_cascade).
- **Evolution lifecycle scenario**: acquire chip → stack to max → evolve → verify evolved effects fire correctly. End-to-end, exercising the chip selection injection system to force the evolution path.
- **NoBump trigger scenario**: bolt hits breaker with no bump input. Verify NoBump trigger fires, effects dispatch correctly, no invariant violations under chaos.
- **Died trigger scenario**: entity dies, Died trigger fires on the dying entity. Verify targeted dispatch, effects on the dead entity resolve correctly, no entity leaks.
- **Impact(Bolt) trigger scenario**: global Impact trigger with Bolt target. Verify dispatch, context entity resolution, no invariant violations.
- **Impacted(Breaker) trigger scenario**: targeted Impacted trigger on breaker. Verify dispatch to both collision participants, context entity correct.
- **Scenario runner refactor**: rename `disallowed_failures`/`allowed_failures` RON fields to `invariants`/`expected_failures` for clarity. Self-tests currently use the confusing convention of listing the same invariant in both fields.

### Out
- New invariant checkers beyond `BoltBirthingLayersZeroed` (existing invariants should cover the rest — verify during implementation)
- Trigger system refactors (this is about adding scenarios, not changing trigger code)
- Coverage for triggers added by future features (new modifiers, protocols, hazards)

## Dependencies
- Depends on: nothing — existing trigger systems and scenario infrastructure are in place
- Blocks: nothing directly, but improves confidence for effect system refactor (todo #3)

## Notes
- Use `chip_selections` injection in scenario RON to force a specific chip evolution path
- NoBump may need a scenario input mode that deliberately avoids bump input (or uses `NeverBump` perfect input mode if it exists)
- Consider whether each new scenario should be `mechanic/` (deterministic, scripted) or `chaos/` (randomized stress)
- Run `reviewer-scenarios` first to get a definitive gap analysis — there may be more gaps than the 4 triggers listed here

## Status
`ready`

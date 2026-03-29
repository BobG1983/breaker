---
name: No-op stub satisfies negative behavior assertions (RED gate violation)
description: Tests asserting "no change occurs" pass immediately against a no-op stub, breaking the RED gate for those behaviors
type: feedback
---

When a spec includes behaviors that assert no state change (e.g., "inactive attraction produces no steering", "entity without component is unaffected", "impact for different bolt is ignored"), the corresponding tests assert that initial/unchanged values are preserved. A no-op stub does nothing — so the initial value IS preserved — and these tests PASS at RED.

This is a RED gate violation: the test cannot distinguish between "stub correctly did nothing" and "production system correctly did nothing."

**Why:** These behaviors are legitimately important to test, but the test setup must ensure the stub would fail them. The fix is to arrange test state so that a WORKING implementation would produce a different result than the INITIAL state. For "should not change" tests, the test must first confirm the system DOES change something in the non-exception case, then verify the exception case is handled. Alternatively, add a second entity that SHOULD be changed, and assert it WAS changed (positive assertion), alongside the negative assertion for the entity under test.

**How to apply:** For every test that asserts a "no change" outcome (velocity unchanged, active=true unchanged, entry not removed), check whether the no-op stub would satisfy that assertion trivially. Flag each as BLOCKING if the test would pass at RED against a stub that does nothing.

Examples seen:
- `apply_attraction_zero_distance_no_steering` — velocity starts at ZERO, stub does nothing, assert ZERO passes
- `apply_attraction_inactive_entry_produces_no_steering` — velocity starts at ZERO, stub does nothing, assert ZERO passes
- `apply_attraction_no_targets_velocity_unchanged` — velocity starts at (100, 200), stub does nothing, assert (100, 200) passes
- `apply_attraction_entity_without_attractions_unaffected` — same
- `apply_attraction_empty_attractions_no_steering` — same
- `manage_attraction_impact_for_different_bolt_is_ignored` — active=true from spawn, stub does nothing, assert active=true passes
- `manage_attraction_all_already_active_reactivation_is_noop` — active=true from spawn, stub does nothing, assert active=true passes
- `fire_count_zero_spawns_no_bolts` (SpawnBolts) — stub spawns nothing, count=0 assertion passes
- `fire_with_timer_already_at_zero_is_idempotent` (TimePenalty) — remaining starts at 0.0, stub does nothing, assert 0.0 passes
- `reverse_at_total_is_idempotent` (TimePenalty) — remaining starts at total=60.0, stub does nothing, assert 60.0 passes
- `fire_does_not_spawn_distance_constraint` (TetherBeam) — stub spawns nothing, count=0 assertion passes
- `reverse_does_not_despawn_previously_spawned_bolts` (SpawnBolts) — fire is also no-op, count_before=count_after=0, assert equal passes
- `fire_spawns_bolts_at_zero_when_owner_has_no_position2d` (TetherBeam) — no bolts spawned, for loop body never runs, vacuous pass
- `reverse_preserves_existing_state` / `reverse_with_state_is_noop` (RandomEffect/EntropyEngine 5C) — spec defines reverse() as a permanent no-op; tests for it trivially pass against ANY stub. Tests for spec-defined no-ops cannot produce RED by definition — flag as BLOCKING when encountered, even if the function is intentionally a no-op per spec.

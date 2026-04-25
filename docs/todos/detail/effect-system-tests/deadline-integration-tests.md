# Deadline: end-to-end test suite (code-driven pipeline)

## Problems addressed

- `audit/protocols/deadline.md` Issue 3 — Zero Deadline-specific integration tests; no test asserts "bolt velocity doubles when timer crosses 25%" end-to-end.
- `audit/protocols/deadline.md` Issue 6 — No test for multi-bolt boost application.
- `audit/protocols/deadline.md` Issue 7 — No test for cleanup at node end.

## Remediation

Create `breaker-game/src/mutators/protocols/deadline/tests/` (new module) with four test files exercising the code-driven Deadline pipeline defined in `deadline-code-driven-effects.md`.

### `threshold_cross_doubles_bolt_velocity.rs`

Build a headless app with the protocol plugin, bolt plugin, and node-timer plugin. Seed the protocol registry with Deadline's RON tuning (`threshold_fraction: 0.25, speed_multiplier: 2.0, damage_multiplier: 2.0`). Activate Deadline via the dispatch message. Spawn one bolt with a known baseline velocity. Drive the node timer down past the 25% threshold. Tick `FixedUpdate` once. Assert the bolt's `EffectStack<SpeedBoostConfig>` carries a `"deadline"`-sourced entry with multiplier 2.0 and its `EffectStack<DamageBoostConfig>` carries the same-sourced entry with multiplier 2.0. Assert bolt velocity magnitude after the next movement tick equals 2.0 \u00d7 baseline.

### `multi_bolt_boost_application.rs`

Activate Deadline. Spawn three bolts simultaneously. Cross the threshold. Assert all three bolts carry `"deadline"`-sourced SpeedBoost and DamageBoost entries with the expected multipliers. Assert none of them double-apply.

### `late_spawn_receives_boost.rs`

Activate Deadline. Spawn one bolt. Cross the threshold (the one bolt picks up the boost). Emit a `BoltSpawned` message for a newly-spawned bolt (simulating Fission or Afterimage's late spawn). Tick `FixedUpdate`. Assert the new bolt ALSO carries the `"deadline"`-sourced SpeedBoost + DamageBoost. Exercises the `deadline_stamp_on_spawn` system.

### `cleanup_at_node_end.rs`

Activate Deadline. Spawn three bolts. Cross the threshold. Verify bolts carry boost entries. Drive `OnExit(NodeState::Playing)`. Assert `deadline_cleanup_on_node_exit` fires and every bolt's `EffectStack<SpeedBoostConfig>` no longer contains `"deadline"`-sourced entries; same for `DamageBoostConfig`. Primary bolts persist past node exit (per `CleanupOnExit<RunState>`); the entries are removed per-stack via `reverse_effect`, not by entity despawn.

Each test uses concrete numeric assertions. No property-based testing. Tests depend on `deadline-code-driven-effects.md` landing first.

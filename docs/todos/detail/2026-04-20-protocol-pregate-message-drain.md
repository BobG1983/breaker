# Protocol pre-gate message drain — cross-cutting refactor

**Status:** ready
**Found during:** Burnout protocol Standard Verification Tier (2026-04-20, reviewer-correctness)
**Severity:** Theoretical bug today — latent for any future feature that toggles `ActiveProtocols` mid-`NodeState::Playing`

## The bug class

Protocols that consume messages (`BumpPerformed`, `BoltImpactCell`, `BoltLost`, `Destroyed<Cell>`, `Dead`, etc.) register their reader systems inside `run_if(protocol_active(ProtocolKind::Foo).and(in_state(NodeState::Playing)))`. When the run-if returns false the whole system is skipped — which means its `MessageReader<T>` cursor never advances, so messages buffered in Bevy 0.18's two-frame double-buffer survive and are processed retroactively the tick the gate opens.

The harness-safe `Option<Res<FooConfig>>` early-return path calls `reader.clear()`, but that path only runs when the system IS scheduled (gate true) and the config happens to be absent. The gate-off path does not.

## Today: unreachable by architecture

Every writer of the relevant messages is itself gated on `NodeState::Playing` (e.g., `breaker::grade_bump`, `bolt::cell_collision`). Since `ActiveProtocols` is mutated only on chip-select → Playing boundaries (never mid-Playing), the only way a protocol's reader can lag its writer across a gate flip is if `ActiveProtocols` is changed while `NodeState::Playing` is live. No gameplay path does that today.

## Tomorrow: reachable if any of these land

- A chip that grants a protocol mid-node
- A hazard that temporarily adds/removes a protocol
- Any state machine that toggles `ActiveProtocols` while `NodeState` stays `Playing`
- An explicit test harness that seeds `ActiveProtocols` after writing messages (exists in test code but the tests never tick to surface it)

## Affected protocols (reviewer-correctness identified)

- `burnout_on_bump`, `burnout_amplify_damage`
- `iron_curtain_on_bolt_lost`
- `siphon_on_cell_destroyed`
- `fracture_on_death`
- `debt_collector` (three systems)
- Likely also `echo_strike`, `fission`, `greed`, `reckless_dash` — audit on implementation

## The fix options

### A — Always-run drain system (preferred)

Add a `drain_readers_when_gated_off` system per protocol that runs UNCONDITIONALLY (no run-if) and clears each reader when the gate is false. Cheap: 1 branch + O(new messages) reads.

### B — Move gate check inside the system body

Drop the `run_if` gate from reader systems; inside each system, check `is_protocol_active` / `in_state` manually and `reader.clear()` + return if gated. Consistent with the existing harness-safe `Option<Res<_>>` pattern.

### C — Accept and document

Add a note to each protocol's `register` doc explaining that mid-Playing `ActiveProtocols` changes may replay stale messages. Acceptable only if no feature ever introduces mid-Playing activation.

## Recommended scope

Audit every protocol's reader systems. Apply Option B uniformly (consistent with harness-safe pattern, no new system registration). Add regression tests that:
1. Write message while gate off
2. Flip gate on
3. Tick
4. Assert no side effect

Start with a shared test helper — `assert_gate_off_messages_not_retroactive<Msg, Side>(...)` — to avoid copy-paste across protocols.

## Test spec sketch (per protocol)

```rust
#[test]
fn buffered_bump_while_inactive_is_not_retroactively_processed() {
    let mut app = build_foo_app(); // config present, protocol NOT seeded
    // no seed_active_protocols_with_foo yet
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // gate off — message buffered, nothing consumed

    seed_active_protocols_with_foo(&mut app, ...);
    tick(&mut app); // gate on — must NOT retroactively consume

    assert_eq!(read_damage_boost_multiplier(&app, bolt), None);
    assert_eq!(count_shockwave_sources(&mut app), 0);
}
```

Repeat for each (protocol, reader-message) pair.

## Priority

Low today (unreachable), but pin now because the moment any feature adds mid-Playing protocol toggling the bug becomes live across 5+ protocols simultaneously. One cross-cutting refactor beats five spot-fixes.

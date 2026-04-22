# Decay

Assumes: #2 (`mutators/hazards/decay/`), #4 (`ApplyTimePenalty` renamed to `ReduceNodeTimer { delta }`).

## What's broken

1. **`ActiveHazards` is read as `Res<ActiveHazards>` instead of `Option<Res<ActiveHazards>>`.** Asymmetric with every other hazard. Harness-fragile: any test that spins up Decay without inserting `ActiveHazards` panics on system-run. Today the `.run_if(hazard_active(Decay))` gate prevents the system from running in that case, so the panic doesn't surface — but the pattern inconsistency is a trap for any future refactor that changes the gating structure.
2. **Behavior 4 ("Paused → no run") has no test.** Pinned only implicitly by the `.run_if(in_state(Playing))` gate. If someone removes or reshapes the gate, the behavior silently breaks.

## Fix

### Param change

`mutators/hazards/decay/decay.rs` (or wherever Decay's tick system lives post-#2) — at the system's `active:` parameter:

```rust
// before
active: Res<ActiveHazards>,

// after
active: Option<Res<ActiveHazards>>,
```

Add an early-return at the top of the body:

```rust
let Some(active) = active else { return };
```

No behavioral change — the `.run_if(hazard_active(Decay))` gate already blocks execution when `ActiveHazards` is absent (since `hazard_active` returns false). The `Option<Res<_>>` wrap makes the system harness-safe in isolation, matching every other hazard's convention.

### Paused-state test

Add to Decay's tests (inline module or separate file — match the current layout; extract to `tests/paused_state.rs` if the current inline module is close to the 400-line threshold).

```rust
#[test]
fn decay_tick_emits_no_reduce_node_timer_when_paused() {
    let mut app = build_test_app();
    app.insert_resource(ActiveHazards::with(HazardKind::Decay, 1));
    app.insert_resource(DecayConfig { base_percent: 15.0, per_level_percent: 5.0 });

    // Transition to Paused.
    transition_node_state(&mut app, NodeState::Paused);

    tick(&mut app);

    let messages = collect_messages::<ReduceNodeTimer>(&mut app);
    assert!(
        messages.is_empty(),
        "Decay must not emit ReduceNodeTimer while paused, got: {messages:?}",
    );
}
```

`ReduceNodeTimer { delta }` is the post-#4 name for the time-penalty message. If this TODO lands before #4, temporarily keep `ApplyTimePenalty` — but the target is `ReduceNodeTimer`.

Match the project's existing paused-state test harness pattern (grep other hazards for `NodeState::Paused` test setups — likely `mutators/hazards/haste/` or similar have a working example).

### Design doc

No design-doc change needed — the behavior is already specified; this just pins the behavior with a test.

## Tests

1. **`decay_tick_emits_no_reduce_node_timer_when_paused`** (body above) — covers the Paused branch.
2. **`decay_tick_no_op_when_inactive_res_missing`** (new) — spin up a headless app WITHOUT inserting `ActiveHazards`; tick the FixedUpdate; assert no panic, no `ReduceNodeTimer` emitted. Pins the `Option<Res<_>>` harness-safety explicitly.
3. **`decay_tick_emits_reduce_node_timer_when_active_and_playing`** (may already exist — verify) — happy-path control.

## Code changes summary

| File | Change |
|------|--------|
| `mutators/hazards/decay/decay.rs` (or wherever Decay's system lives) | `active: Res<ActiveHazards>` → `active: Option<Res<ActiveHazards>>`; add `let Some(active) = active else { return };` at body top |
| `mutators/hazards/decay/tests/paused_state.rs` (new file, or inline) | Tests 1-2 from above |

## Out of scope

- Decay's damage/timer mechanics — unchanged.
- Decay RON tuning — `ron-tuning-values.md`.
- Any other hazard's `Res<ActiveHazards>` → `Option<Res<_>>` conversion — audit separately; if another hazard has the same pattern, convert it in its own small TODO or fold into this one during impl. Decay's the only confirmed offender.

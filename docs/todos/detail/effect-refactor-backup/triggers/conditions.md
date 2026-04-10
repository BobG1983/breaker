# Conditions (for During)

Conditions are state-based (not event-based). They have a start and end. `During(condition, ...)` fires effects on condition start and reverses them on condition end. Conditions can cycle (start/end/start/end...).

## Conditions
- `NodeActive` — node is playing or paused
- `ShieldActive` — at least one ShieldWall entity exists
- `ComboActive(u32)` — N consecutive perfect bumps achieved

## Condition Monitor Systems

### `monitor_node_active`
**Schedule**: FixedUpdate (always — not gated by NodeState)

```
fn monitor_node_active(node_state: Res<State<NodeState>>, ...) {
    // NodeActive = Playing or Paused (spans both)
    // Start: transition to Playing from non-Playing non-Paused state
    // End: node teardown (exit from Playing/Paused entirely)
    
    // On start: for each entity with During(NodeActive, ...) in BoundEffects:
    //   - Direct Fire: execute effect (it's reversible)
    //   - Nested When: register inner When into BoundEffects.triggers with scope source
    
    // On end: for each entity with During(NodeActive, ...) in BoundEffects:
    //   - Direct Fire: reverse the effect
    //   - Nested When: unregister from BoundEffects.triggers by scope source
    //     + clean up StagedEffects entries armed from it
}
```

### `monitor_shield_active`
**Schedule**: FixedUpdate

```
fn monitor_shield_active(
    added: Query<Entity, Added<ShieldWall>>,
    removed: RemovedComponents<ShieldWall>,
    existing: Query<Entity, With<ShieldWall>>,
    ...
) {
    // Start: Added<ShieldWall> detected AND no shield was active before
    // End: ShieldWall removed AND no ShieldWall entities remain
    // Same activate/deactivate pattern as NodeActive
}
```

### `monitor_combo_active`
**Schedule**: FixedUpdate

```
fn monitor_combo_active(
    bump_events: MessageReader<BumpGraded>,
    tracker: Res<HighlightTracker>,  // has consecutive_perfect_bumps
    ...
) {
    // For each ComboActive(n) condition in BoundEffects:
    // Start: consecutive_perfect_bumps crosses n upward
    // End: consecutive_perfect_bumps resets to 0 (non-perfect bump)
    // Must track per-n threshold state to detect crossings
}
```

## During Lifecycle (how conditions interact with effects)

### Direct Fire: `During(NodeActive, Fire(SpeedBoost(1.5)))`
- **Activate**: fire SpeedBoost(1.5) on entity
- **Deactivate**: reverse SpeedBoost(1.5) on entity
- Condition cycles: re-fire on next activation, re-reverse on next deactivation

### Direct Sequence: `During(NodeActive, Sequence([Fire(SpeedBoost), Fire(DamageBoost)]))`
- **Activate**: fire all children in order
- **Deactivate**: reverse all children in reverse order

### Nested When: `During(NodeActive, When(PerfectBumped, Fire(Explode)))`
- **Activate**: register `When(PerfectBumped, Fire(Explode))` into BoundEffects.triggers
  with scope source `"ChipName:During(NodeActive)"`
- **While active**: normal trigger dispatch handles PerfectBumped → Fire(Explode)
- **Deactivate**: unregister from BoundEffects.triggers by scope source
  + clean up any StagedEffects entries armed from it

## Scope Source
Derived from chip SourceId + condition: `format!("{source}:During({condition:?})")`
Enables targeted cleanup of just the During-registered entries without affecting other entries from the same chip.

## Notes
- Conditions are first-class — not desugared into synthetic triggers (Decision #3, #14)
- BoundEffects has separate `conditions` Vec (not mixed with triggers Vec)
- Conditions can cycle — fire/reverse/fire/reverse as state changes
- `ComboActive(n)` needs per-threshold state tracking to detect crossings

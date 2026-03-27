# System Ordering

All effect systems run in `FixedUpdate`. The ordering chain:

```
Collision systems (bolt, breaker, cells domains)
    → send impact messages (BoltImpactCell, etc.)
    ↓
Trigger bridge systems (effect/triggers/)
    → walk chains, queue FireEffectCommand / ReverseEffectCommand / TransferCommand
    ↓
Until desugaring system
    → finds Until nodes, fires Do children, pushes When children to BoundEffects,
      replaces Until with When+Reverse in StagedEffects
    ↓
Timer system (effect/triggers/timer.rs)
    → ticks TimeExpires entries in StagedEffects, fires Reverse when expired
    ↓
apply_deferred
    → FireEffectCommand / ReverseEffectCommand / TransferCommand execute with &mut World
    ↓
Effect runtime systems (effect/effects/)
    → apply_speed_boosts, tick_shockwave, shockwave_collision, etc.
```

## Ordering Rules

- **Trigger bridges** run `.after()` the collision/game systems that produce their messages (e.g., `bridge_impact_bolt_cell.after(BoltSystems::CellCollision)`)
- **Trigger bridges for the same message** can run in parallel — impact and impacted both read BoltImpactCell but don't write-conflict
- **Until desugaring** runs after all trigger bridges
- **Timer system** runs after all trigger bridges (parallel with Until desugaring is fine — they touch different node types)
- **Effect runtime systems** (recalculation, tick, collision) run after `apply_deferred` so they see the effects that were just fired

# System Set Ordering

## Within the effect domain

```
EffectSystems::Bridge
    ↓ (after)
EffectSystems::Tick
    ↓ (after)
EffectSystems::Conditions
```

- **Bridge before Tick**: bridges dispatch triggers and fire effects (via commands). Effect entities are spawned after command flush. Tick systems need those entities to exist.
- **Tick before Conditions**: tick systems may spawn or despawn shield walls, which affects ShieldActive. Condition evaluation must see the final state.
- **Reset**: runs on `OnEnter(NodeState)`, not in the FixedUpdate chain. No ordering relationship with Bridge/Tick/Conditions.

## Special case: timer systems

`tick_effect_timers` and `check_node_timer_thresholds` live in `EffectSystems::Tick` but produce messages consumed by bridge systems (`on_time_expires`, `on_node_timer_threshold_occurred`). Since Tick runs AFTER Bridge, these messages are processed next frame by the bridges. This is acceptable — timer expiry and threshold crossing take effect one frame after detection.

## External dependencies (non-effect system sets)

### Must run AFTER

| External system set | Why | Which effect systems depend on it |
|-------------------|-----|----------------------------------|
| `BoltSystems::CellCollision` | Produces `BoltImpactCell`, `DamageDealt<Cell>` | `on_impacted` (bolt-cell), `on_impact_occurred` (bolt-cell) |
| `BoltSystems::WallCollision` | Produces `BoltImpactWall` | `on_impacted` (bolt-wall), `on_impact_occurred` (bolt-wall) |
| `BoltSystems::BreakerCollision` | Produces `BoltImpactBreaker` | `on_impacted` (bolt-breaker), `on_no_bump_occurred` |
| `BoltSystems::BoltLost` | Produces `BoltLost` | `on_bolt_lost_occurred` |
| `BreakerSystems::GradeBump` | Produces `BumpPerformed`, `BumpWhiffed` | All bump bridges |
| `DeathPipelineSystems::ApplyDamage` | Must process damage before death detection | Not a direct ordering dependency — death bridges read `Destroyed<T>` from the previous frame via standard Bevy message persistence |

Note: Death bridges (`on_destroyed::<T>`) read `Destroyed<T>` messages from the **previous frame**. They do NOT need to run after the death pipeline in the same frame. This is the standard Bevy message pattern. Death-triggered effects have a one-frame delay, which is acceptable at 60fps.

### Must run BEFORE

| External system set | Why | Which effect systems must precede it |
|-------------------|-----|-------------------------------------|
| `PostFixedUpdate::process_despawn_requests` | Entities must survive through bridge + tick + conditions | All effect systems complete before despawn |

## Parallelism within sets

### Bridge set
All bridge systems CAN run in parallel — they read different messages and call walk_effects on different entities. No shared mutable state between bridges.

### Tick set
Most tick systems CAN run in parallel — they operate on different component populations (ShockwaveSource vs ChainLightningChain vs PulseEmitter etc.). Exceptions are the chained pipelines listed in system-sets.md which must run sequentially within their chain.

### Conditions set
Single system — no parallelism concern.

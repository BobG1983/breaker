# Plugin Wiring

How `EffectPlugin` registers system sets, systems, and ordering constraints.

## System Sets

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum EffectSystems {
    /// Bridge systems that translate game events to trigger dispatches.
    /// Each bridge reads a game event message, builds TriggerContext, and
    /// calls the walking algorithm on scoped entities.
    Bridge,

    /// Effect-specific runtime systems (tick, damage, cleanup).
    /// These advance spawned effect entities each frame.
    Tick,

    /// Reset systems that run on node start.
    /// These clear per-node accumulators and counters.
    Reset,
}
```

## System Registration

### Bridge set

All bridge systems run in `FixedUpdate`, in the `EffectSystems::Bridge` set, **after** the game systems that produce the messages they consume.

| System | Message consumed | Runs after |
|--------|-----------------|------------|
| `on_bumped` | BumpResult | bump detection systems |
| `on_perfect_bumped` | BumpResult | bump detection systems |
| `on_early_bumped` | BumpResult | bump detection systems |
| `on_late_bumped` | BumpResult | bump detection systems |
| `on_bump_occurred` | BumpResult | bump detection systems |
| `on_perfect_bump_occurred` | BumpResult | bump detection systems |
| `on_early_bump_occurred` | BumpResult | bump detection systems |
| `on_late_bump_occurred` | BumpResult | bump detection systems |
| `on_bump_whiff_occurred` | BumpWhiff | bump detection systems |
| `on_no_bump_occurred` | NoBumpOccurred | bump detection systems |
| `on_impacted` | BoltImpactCell | collision systems |
| `on_impact_occurred` | BoltImpactCell | collision systems |
| `on_bolt_lost_occurred` | BoltLost | bolt lost detection |
| `on_node_start_occurred` | (OnEnter schedule or message) | node state transition |
| `on_node_end_occurred` | (OnExit schedule or message) | node state transition |
| `on_node_timer_threshold_occurred` | NodeTimerThresholdCrossed | `check_node_timer_thresholds` |
| `on_time_expires` | EffectTimerExpired | `tick_effect_timers` |

See `creating-triggers/trigger-api/bridge-systems.md` for the bridge pattern.
See `migration/new-trigger-implementations/` for per-trigger behavioral specs.

### Tick set

All tick systems run in `FixedUpdate`, in the `EffectSystems::Tick` set, **after** `EffectSystems::Bridge`.

| System | Purpose |
|--------|---------|
| `tick_shockwave` | Expand shockwave radius each frame |
| `sync_shockwave_visual` | Sync visual mesh/sprite to current radius |
| `apply_shockwave_damage` | Damage entities overlapping the shockwave |
| `despawn_finished_shockwave` | Despawn shockwaves at max radius |
| `tick_chain_lightning` | Propagate chain lightning arcs |
| `tick_anchor` | Tick anchor lock/unlock state |
| `apply_attraction` | Apply attraction forces to bolts |
| `tick_pulse` | Tick pulse emitter cooldown, fire when ready |
| `tick_shield_duration` | Count down shield lifetime |
| `tick_phantom_lifetime` | Count down phantom bolt lifetime |
| `tick_tether_beam_damage` | Apply tether beam damage per tick |
| `cleanup_tether_beams` | Remove tether beams whose target is despawned |
| `tick_gravity_wells` | Apply gravity well forces |
| `despawn_expired_wells` | Despawn gravity wells past their duration |
| `tick_effect_timers` | Tick EffectTimers, emit EffectTimerExpired messages |
| `check_node_timer_thresholds` | Check node timer against threshold registry, emit NodeTimerThresholdCrossed |

### Reset set

Reset systems run on `OnEnter(NodeState::Running)` (or equivalent node-start schedule), in the `EffectSystems::Reset` set.

| System | Purpose |
|--------|---------|
| `reset_ramping_damage` | Zero out RampingDamageAccumulator on node start |
| `reset_entropy_counter` | Zero out EntropyCounter on node start |

## Ordering Within Sets

Most systems within a set have no ordering constraints relative to each other. The exceptions are chained pipelines where one system's output feeds the next:

### Shockwave pipeline (chained)

```
tick_shockwave
  -> sync_shockwave_visual
  -> apply_shockwave_damage
  -> despawn_finished_shockwave
```

`tick_shockwave` expands the radius. `sync_shockwave_visual` reads the updated radius to position the visual. `apply_shockwave_damage` uses the expanded radius for overlap checks. `despawn_finished_shockwave` removes completed shockwaves after damage has been applied.

### Tether beam pipeline (chained)

```
tick_tether_beam_damage
  -> cleanup_tether_beams
```

`tick_tether_beam_damage` applies damage first. `cleanup_tether_beams` removes beams whose targets were despawned (possibly as a result of the damage).

### Gravity well pipeline (chained)

```
tick_gravity_wells
  -> despawn_expired_wells
```

`tick_gravity_wells` applies forces first. `despawn_expired_wells` removes expired wells after their final frame of influence.

### Timer -> bridge ordering

```
tick_effect_timers                -> on_time_expires (Bridge set)
check_node_timer_thresholds       -> on_node_timer_threshold_occurred (Bridge set)
```

`tick_effect_timers` and `check_node_timer_thresholds` emit messages that their corresponding bridge systems consume. These two tick systems run in the Tick set but must be ordered **before** the Bridge set for same-frame dispatch. Since Tick runs after Bridge by default, these two systems are the exception: they run in `EffectSystems::Tick` but with an explicit `.before(EffectSystems::Bridge)` constraint, or alternatively they are placed in a dedicated pre-bridge phase. The implementation must ensure their messages are available in the same frame.

**Alternative**: Place `tick_effect_timers` and `check_node_timer_thresholds` in the Bridge set itself, ordered before the bridge systems that consume their messages. Either approach works; the constraint is that the message must be readable in the same frame it is emitted.

### All other systems

No intra-set ordering needed. They operate on independent entity populations or components with no same-frame data dependencies.

## Death Bridge Systems

Death bridge systems (`bridge_destroyed<T>` for Cell, Wall, Bolt) are **NOT** registered by `EffectPlugin`. They are registered by the unified death pipeline plugin, which owns the full damage -> death -> trigger chain.

See:
- `docs/architecture/effects/death_pipeline.md` -- unified death pipeline architecture
- `docs/todos/detail/effect-refactor/dispatching-triggers/death/` -- death trigger dispatch specs

This separation exists because death bridges are tightly coupled to the death pipeline's message flow (`Destroyed<T>`) and must be ordered within that pipeline's system chain, not within the effect plugin's Bridge set.

## Set Ordering Summary

```
FixedUpdate:
  [game systems: bump detection, collision, bolt lost, ...]
    |
    v
  EffectSystems::Bridge  (bridges read game messages, dispatch triggers, walk trees)
    |
    v
  EffectSystems::Tick    (tick spawned effect entities, emit timer messages)
    |
    v
  [death pipeline: apply_damage -> detect_deaths -> domain handlers -> bridge_destroyed]

OnEnter(NodeState::Running):
  EffectSystems::Reset   (zero accumulators/counters for the new node)
```

Note: `tick_effect_timers` and `check_node_timer_thresholds` need special ordering to ensure their messages reach bridge systems in the same frame. See "Timer -> bridge ordering" above.

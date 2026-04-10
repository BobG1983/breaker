# System Sets

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum EffectSystems {
    Bridge,
    Tick,
    Conditions,
    Reset,
}
```

## EffectSystems::Bridge

Bridge systems that translate game events to trigger dispatches. Each reads a message, builds TriggerContext, and calls `walk_effects`.

| System | Message consumed |
|--------|-----------------|
| `on_bumped` | `BumpPerformed` |
| `on_perfect_bumped` | `BumpPerformed` |
| `on_early_bumped` | `BumpPerformed` |
| `on_late_bumped` | `BumpPerformed` |
| `on_bump_occurred` | `BumpPerformed` |
| `on_perfect_bump_occurred` | `BumpPerformed` |
| `on_early_bump_occurred` | `BumpPerformed` |
| `on_late_bump_occurred` | `BumpPerformed` |
| `on_bump_whiff_occurred` | `BumpWhiffed` |
| `on_no_bump_occurred` | `BoltImpactBreaker` (where `BumpStatus::Inactive`) |
| `on_impacted` (×6 sub-systems) | Collision messages |
| `on_impact_occurred` (×6 sub-systems) | Collision messages |
| `on_bolt_lost_occurred` | `BoltLost` |
| `on_node_start_occurred` | Node state transition |
| `on_node_end_occurred` | Node state transition |
| `on_node_timer_threshold_occurred` | `NodeTimerThresholdCrossed` |
| `on_time_expires` | `EffectTimerExpired` |
| `bridge_destroyed::<Cell>` | `Destroyed<Cell>` |
| `bridge_destroyed::<Bolt>` | `Destroyed<Bolt>` |
| `bridge_destroyed::<Wall>` | `Destroyed<Wall>` |
| `bridge_destroyed::<Breaker>` | `Destroyed<Breaker>` |

## EffectSystems::Tick

Runtime systems for spawned effect entities. Advance state each frame.

| System | Purpose | Chained with |
|--------|---------|-------------|
| `tick_shockwave` | Expand radius | → sync_shockwave_visual → apply_shockwave_damage → despawn_finished_shockwave |
| `sync_shockwave_visual` | Sync visual to radius | (chained) |
| `apply_shockwave_damage` | Damage cells in radius | (chained) |
| `despawn_finished_shockwave` | Despawn at max radius | (chained) |
| `tick_chain_lightning` | Advance arc state machine | |
| `tick_anchor` | Anchor plant/unplant state | |
| `apply_attraction` | Steer toward targets | |
| `tick_pulse` | Emit periodic shockwaves | |
| `tick_shield_duration` | Count down shield lifetime | |
| `tick_phantom_lifetime` | Count down phantom lifetime | |
| `tick_tether_beam_damage` | Damage cells along beam | → cleanup_tether_beams |
| `cleanup_tether_beams` | Remove beams with missing endpoints | (chained) |
| `tick_gravity_wells` | Pull bolts toward center | → despawn_expired_wells |
| `despawn_expired_wells` | Despawn expired wells | (chained) |
| `tick_effect_timers` | Tick EffectTimers, send EffectTimerExpired | |
| `check_node_timer_thresholds` | Check timer ratio, send NodeTimerThresholdCrossed | |

## EffectSystems::Conditions

Condition monitoring for During nodes.

| System | Purpose |
|--------|---------|
| `evaluate_conditions` | Poll NodeActive/ShieldActive/ComboActive, fire/reverse During entries on transitions |

## EffectSystems::Reset

Per-node reset systems. Run on `OnEnter(NodeState)`, not in FixedUpdate.

| System | Purpose |
|--------|---------|
| `reset_ramping_damage` | Zero out RampingDamageAccumulator |
| `reset_entropy_counter` | Zero out EntropyCounter |

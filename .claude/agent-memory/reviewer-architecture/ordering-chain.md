---
name: Ordering Chain
description: Current system ordering chain for FixedUpdate and OnEnter(Playing)
type: reference
---

## Current Ordering Chain (verified 2026-03-20)

### Defined SystemSet Variants (code reality)
| Set | Domain | Tags | Cross-Domain Consumers |
|-----|--------|------|----------------------|
| BreakerSystems::Move | breaker/sets.rs | move_breaker | bolt (hover_bolt, prepare_bolt_velocity) |
| BreakerSystems::InitParams | breaker/sets.rs | init_breaker_params | behaviors (init_archetype), bolt (init_bolt_params not directly, but via ordering) |
| BreakerSystems::Reset | breaker/sets.rs | reset_breaker | bolt (reset_bolt) |
| BreakerSystems::GradeBump | breaker/sets.rs | grade_bump | behaviors (bridge_bump, bridge_bump_whiff) |
| BoltSystems::InitParams | bolt/sets.rs | init_bolt_params | (intra-domain: reset_bolt) |
| BoltSystems::PrepareVelocity | bolt/sets.rs | prepare_bolt_velocity | physics (bolt_cell_collision) |
| BoltSystems::Reset | bolt/sets.rs | reset_bolt | (no cross-domain consumers) |
| PhysicsSystems::BreakerCollision | physics/sets.rs | bolt_breaker_collision | breaker (grade_bump) |
| PhysicsSystems::BoltLost | physics/sets.rs | bolt_lost | behaviors (bridge_bolt_lost) |
| BehaviorSystems::Bridge | behaviors/sets.rs | bridge_bump, bridge_bolt_lost, bridge_bump_whiff, bridge_cell_impact, bridge_breaker_impact, bridge_wall_impact, bridge_cell_destroyed | bolt (spawn_additional_bolt) |
| UiSystems::SpawnTimerHud | ui/sets.rs | spawn_timer_hud | behaviors (spawn_lives_display) |
| NodeSystems::Spawn | run/node/sets.rs | spawn_cells_from_layout | breaker (apply_entity_scale_to_breaker), bolt (apply_entity_scale_to_bolt) |
| NodeSystems::TrackCompletion | run/node/sets.rs | track_node_completion | run (handle_node_cleared) |
| NodeSystems::TickTimer | run/node/sets.rs | tick_node_timer | (intra-domain: apply_time_penalty) |
| NodeSystems::ApplyTimePenalty | run/node/sets.rs | apply_time_penalty | run (handle_timer_expired) |
| NodeSystems::InitTimer | run/node/sets.rs | init_node_timer | (no cross-domain consumers) |

### FixedUpdate Chain
```
BreakerSystems::Move
  <- (hover_bolt, prepare_bolt_velocity) .after(BreakerSystems::Move)
    BoltSystems::PrepareVelocity
      <- bolt_cell_collision .after(BoltSystems::PrepareVelocity)
        <- bolt_breaker_collision .after(bolt_cell_collision)
          PhysicsSystems::BreakerCollision
            <- grade_bump .after(update_bump)
                          .after(PhysicsSystems::BreakerCollision)
                          .in_set(BreakerSystems::GradeBump)
              <- (perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text) .after(grade_bump)
            <- clamp_bolt_to_playfield .after(bolt_breaker_collision)
            <- bolt_lost .after(clamp_bolt_to_playfield)
              PhysicsSystems::BoltLost
                <- bridge_bolt_lost .after(PhysicsSystems::BoltLost)
                   .in_set(BehaviorSystems::Bridge)
            <- bridge_bump .after(BreakerSystems::GradeBump)
               .in_set(BehaviorSystems::Bridge)
            <- bridge_bump_whiff .after(BreakerSystems::GradeBump)
               .in_set(BehaviorSystems::Bridge)
              BehaviorSystems::Bridge
                <- spawn_additional_bolt .after(BehaviorSystems::Bridge)

NodeSystems::TrackCompletion
  <- handle_node_cleared .after(NodeSystems::TrackCompletion)
NodeSystems::TickTimer
  <- apply_time_penalty .after(NodeSystems::TickTimer)
    NodeSystems::ApplyTimePenalty
      <- handle_timer_expired .after(NodeSystems::ApplyTimePenalty)
                               .after(handle_node_cleared)
        <- handle_run_lost .after(handle_node_cleared)
                            .after(handle_timer_expired)
```

### OnEnter(Playing) Init Chain
```
apply_archetype_config_overrides .before(BreakerSystems::InitParams)
spawn_breaker → init_breaker_params .in_set(BreakerSystems::InitParams)
  apply_entity_scale_to_breaker .after(BreakerSystems::InitParams) .after(NodeSystems::Spawn)
  reset_breaker .after(BreakerSystems::InitParams) .in_set(BreakerSystems::Reset)
spawn_bolt → init_bolt_params .in_set(BoltSystems::InitParams)
  apply_entity_scale_to_bolt .after(BoltSystems::InitParams) .after(NodeSystems::Spawn)
  reset_bolt .after(BoltSystems::InitParams) .after(BreakerSystems::Reset) .in_set(BoltSystems::Reset)
init_archetype .after(BreakerSystems::InitParams)
spawn_side_panels → ApplyDeferred → spawn_timer_hud .in_set(UiSystems::SpawnTimerHud)
spawn_lives_display .after(init_archetype) .after(UiSystems::SpawnTimerHud)
set_active_layout → spawn_cells_from_layout .in_set(NodeSystems::Spawn) → init_clear_remaining → init_node_timer .in_set(NodeSystems::InitTimer)  [chained]
```

### OnExit(MainMenu)
```
reset_run_state
  <- generate_node_sequence_system .after(reset_run_state)
```

### Breaker Intra-Domain
```
update_bump → move_breaker → update_breaker_state
grade_bump .after(update_bump) .after(PhysicsSystems::BreakerCollision)
trigger_bump_visual .after(update_bump)
Update schedule: animate_bump_visual, animate_tilt_visual, width_boost_visual
```

### Known Doc Drift
- ordering.md OnEnter(Playing) chain is missing apply_entity_scale_to_breaker and apply_entity_scale_to_bolt (both in breaker/plugin.rs OnEnter(Playing) .after(BreakerSystems::InitParams).after(NodeSystems::Spawn))
- NodeSystems::Spawn table entry does not list cross-domain consumers (breaker, bolt entity scale systems)

Note: BoltSystems::InitParams/Reset, DamageCell, BoltHitWall, and BoltHitBreaker consumer list were all fixed in docs as of 2026-03-21/2026-03-22.

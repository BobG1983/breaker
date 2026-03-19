---
name: Ordering Chain
description: Current system ordering chain for FixedUpdate and OnEnter(Playing)
type: reference
---

## Current Ordering Chain (verified 2026-03-16)
```
BreakerSystems::Move
  <- (hover_bolt, prepare_bolt_velocity) .after(BreakerSystems::Move)
    BoltSystems::PrepareVelocity
      <- bolt_cell_collision .after(BoltSystems::PrepareVelocity)
        <- bolt_breaker_collision .after(bolt_cell_collision)
          PhysicsSystems::BreakerCollision
            <- apply_bump_velocity .after(PhysicsSystems::BreakerCollision)
                                   .before(PhysicsSystems::BoltLost)
            <- grade_bump .after(update_bump)
                          .after(PhysicsSystems::BreakerCollision)
              <- (perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text) .after(grade_bump)
            <- bolt_lost .after(bolt_breaker_collision)
              PhysicsSystems::BoltLost
                <- bridge_bolt_lost .after(PhysicsSystems::BoltLost)
                   .in_set(BehaviorSystems::Bridge)
            <- bridge_bump .after(PhysicsSystems::BreakerCollision)
               .in_set(BehaviorSystems::Bridge)
              BehaviorSystems::Bridge
                <- spawn_additional_bolt .after(BehaviorSystems::Bridge)
  NodeSystems::TickTimer
    <- apply_time_penalty .after(NodeSystems::TickTimer)
```

## OnEnter(Playing) Init Chain
```
apply_archetype_config_overrides .before(BreakerSystems::InitParams)
init_breaker_params .in_set(BreakerSystems::InitParams)
init_archetype .after(BreakerSystems::InitParams)
spawn_timer_hud .in_set(UiSystems::SpawnTimerHud)
spawn_lives_display .after(init_archetype) .after(UiSystems::SpawnTimerHud)
```

## Breaker Intra-Domain
```
update_bump → move_breaker → update_breaker_state → grade_bump
trigger_bump_visual .after(update_bump)
Update schedule: animate_bump_visual, animate_tilt_visual
```

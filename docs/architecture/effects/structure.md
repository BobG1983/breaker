# Domain Structure

```
effect_v3/
  mod.rs
  plugin.rs              — EffectV3Plugin, registers all 30 effect configs (Fireable::register) and all trigger categories
  sets.rs                — EffectV3Systems set (Bridge, Tick, Conditions, Reset variants)
  commands.rs             — EffectCommandsExt trait, FireEffectCommand, ReverseEffectCommand, RouteCommand, PushBoundEffects
  core/
    types/                — directory module (split from types.rs)
      definitions/        — directory module (split from definitions.rs for fire/reverse line count)
        enums.rs          — EffectKind enum, Trigger, EffectNode (with Reverse variant),
                            Target, RootEffect, ImpactTarget, AttractionType, BoundEffects, StagedEffects, EffectSourceChip
        fire.rs           — EffectKind::fire() and 3 private helpers (fire_aoe_and_spawn, fire_utility_and_spawn, fire_breaker_effects)
        reverse.rs        — EffectKind::reverse() and 3 private helpers (reverse_aoe_and_spawn, reverse_utility, reverse_breaker_effects)
  triggers/
    mod.rs                — pub mod declarations + register() dispatcher
    bump/                 — Directory module: bump trigger bridges
    impact/               — Directory module: impact trigger bridges
    death/                — Directory module: death trigger bridges (on_destroyed::<T> generic)
    bolt_lost/            — Directory module: bolt lost trigger bridges
    node/                 — Directory module: node lifecycle bridges + check_node_timer_thresholds
    time/                 — Directory module: tick_effect_timers + on_time_expires
  effects/
    mod.rs                — register() calling each effect's register()
    <name>.rs or <name>/  — one per effect: fire(), reverse(), register(), components, runtime systems
```

## What Lives Where

**Effect domain** (`effect_v3/`):
- Core types (EffectKind enum, Trigger enum, EffectNode, etc.)
- Commands extension
- Trigger bridge systems
- Effect fire/reverse functions and runtime systems

**Dispatch pipeline** (`effect_v3/dispatch/` or co-located):
- `death_bridge.rs` — `bridge_destroyed<T: GameEntity>` system
- Command extensions (`effect/commands.rs`) — `EffectCommandsExt` trait implementing EntityCommand for fire/reverse/stamp/route/equip/unequip

**Outside effect domain**:
- Dispatch lives in entity domains (`chips/`, `breaker/`, `cells/`)
- Collision detection lives in entity domains (`bolt/`, `breaker/`, `cells/`)
- Impact messages are defined in the detecting domain

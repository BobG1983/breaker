# Domain Structure

```
effect/
  mod.rs
  plugin.rs              — EffectPlugin, registers all effects and triggers
  commands.rs             — EffectCommandsExt trait, FireEffectCommand, ReverseEffectCommand, TransferCommand, PushBoundEffects
  core/
    types/                — directory module (split from types.rs)
      definitions/        — directory module (split from definitions.rs for fire/reverse line count)
        enums.rs          — EffectKind enum, Trigger, EffectNode (with Reverse variant),
                            Target, RootEffect, ImpactTarget, AttractionType, BoundEffects, StagedEffects, EffectSourceChip
        fire.rs           — EffectKind::fire() and 3 private helpers (fire_aoe_and_spawn, fire_utility_and_spawn, fire_breaker_effects)
        reverse.rs        — EffectKind::reverse() and 3 private helpers (reverse_aoe_and_spawn, reverse_utility, reverse_breaker_effects)
  triggers/
    mod.rs                — register() calling each trigger's register()
    <name>.rs or <name>/  — one per trigger: register(), bridge system (dir module when tests present)
  effects/
    mod.rs                — register() calling each effect's register(); spawn_extra_bolt helper
    <name>.rs or <name>/  — one per effect: fire(), reverse(), register(), components, runtime systems
```

## What Lives Where

**Effect domain** (`effect/`):
- Core types (EffectKind enum, Trigger enum, EffectNode, etc.)
- Commands extension
- Trigger bridge systems
- Effect fire/reverse functions and runtime systems

**Outside effect domain**:
- Dispatch lives in entity domains (`chips/`, `breaker/`, `cells/`)
- Collision detection lives in entity domains (`bolt/`, `breaker/`, `cells/`)
- Impact messages are defined in the detecting domain

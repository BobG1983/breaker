# Domain Structure

```
effect/
  mod.rs
  plugin.rs              — EffectPlugin, registers all effects and triggers
  commands.rs             — EffectCommandsExt trait, FireEffectCommand, ReverseEffectCommand, TransferCommand, PushBoundEffects
  core/
    types/                — directory module (split from types.rs)
      definitions.rs      — EffectKind enum (with fire/reverse methods), Trigger, EffectNode (with Reverse variant),
                            Target, RootEffect, ImpactTarget, AttractionType, BoundEffects, StagedEffects, EffectSourceChip
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

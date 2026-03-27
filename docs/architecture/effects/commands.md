# Commands Extension

File: `effect/commands.rs`

## EffectCommandsExt Trait

```rust
pub trait EffectCommandsExt {
    fn fire_effect(&mut self, entity: Entity, effect: EffectKind);
    fn reverse_effect(&mut self, entity: Entity, effect: EffectKind);
    fn transfer_effect(&mut self, entity: Entity, children: Vec<EffectNode>, permanent: bool);
}
```

Implemented on Bevy's `Commands`. Each method queues a custom `Command` that gets `&mut World` when applied at the next `apply_deferred` sync point.

## Custom Commands

**`FireEffectCommand`** — calls `effect.fire(entity, world)` via `EffectKind::fire()`/`EffectKind::reverse()`. The effect handler gets full world access to query components, spawn entities, insert/remove components.

**`ReverseEffectCommand`** — calls `effect.reverse(entity, world)` via `EffectKind::fire()`/`EffectKind::reverse()`. The reverse handler undoes whatever `fire()` did.

**`TransferCommand`** — pushes children to the target entity's StagedEffects (default) or BoundEffects (if `permanent: true`). Bare Do children in the transfer are fired directly on the target entity via `commands.fire_effect()`.

## Why Commands

Trigger systems are **normal Bevy systems** — they don't need exclusive world access. They read BoundEffects/StagedEffects via queries, pattern-match on nodes, and queue commands. The actual effect execution (which needs to query arbitrary components) happens later via the Command's `apply(self, world: &mut World)`.

This means trigger evaluation doesn't block parallelism. Effects get full `&mut World` access only at apply time.

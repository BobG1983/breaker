# Commands Extension

File: `effect/commands.rs`

## EffectCommandsExt Trait

```rust
pub trait EffectCommandsExt {
    fn fire_effect(&mut self, entity: Entity, effect: EffectKind, source_chip: String);
    fn reverse_effect(&mut self, entity: Entity, effect: EffectKind, source_chip: String);
    fn transfer_effect(&mut self, entity: Entity, chip_name: String, children: Vec<EffectNode>, permanent: bool);
}
```

Implemented on Bevy's `Commands`. Each method queues a custom `Command` that gets `&mut World` when applied at the next `apply_deferred` sync point.

The `source_chip` / `chip_name` parameter carries chip attribution for damage tracking. It is forwarded to `EffectKind::fire()`/`reverse()` and ultimately into `DamageCell.source_chip` for AoE effects.

## Custom Commands

**`FireEffectCommand`** — carries `entity`, `effect: EffectKind`, and `source_chip: String`. Calls `effect.fire(entity, &source_chip, world)` at apply time. The effect handler gets full world access to query components, spawn entities, insert/remove components.

**`ReverseEffectCommand`** — carries `entity`, `effect: EffectKind`, and `source_chip: String`. Calls `effect.reverse(entity, &source_chip, world)` at apply time. The reverse handler undoes whatever `fire()` did.

**`TransferCommand`** — pushes non-`Do` children to the target entity's `StagedEffects` (default) or `BoundEffects` (if `permanent: true`). `Do` children in the transfer are fired directly on the target entity via `effect.fire(entity, &chip_name, world)`.

## Why Commands

Trigger systems are **normal Bevy systems** — they don't need exclusive world access. They read BoundEffects/StagedEffects via queries, pattern-match on nodes, and queue commands. The actual effect execution (which needs to query arbitrary components) happens later via the Command's `apply(self, world: &mut World)`.

This means trigger evaluation doesn't block parallelism. Effects get full `&mut World` access only at apply time.

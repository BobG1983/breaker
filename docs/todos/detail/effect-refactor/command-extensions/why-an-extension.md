# Why a Command Extension

## The Pattern

A command extension is a trait on `Commands` where each method packages its arguments into a struct and calls `self.queue()`. The struct implements `Command` with an `apply(self, world: &mut World)` method that does the actual work.

```rust
pub trait EffectCommandsExt {
    fn stamp_effect(&mut self, entity: Entity, source: String, tree: Tree);
}

impl EffectCommandsExt for Commands<'_, '_> {
    fn stamp_effect(&mut self, entity: Entity, source: String, tree: Tree) {
        self.queue(StampEffectCommand { entity, source, tree });
    }
}

struct StampEffectCommand { entity: Entity, source: String, tree: Tree }

impl Command for StampEffectCommand {
    fn apply(self, world: &mut World) {
        // exclusive world access here
    }
}
```

## Why

Commands are deferred — they queue work that executes at the next sync point when Bevy flushes commands. This means:

- Systems that call `commands.fire_effect(...)` don't need exclusive world access
- Multiple systems can queue effects in the same frame without conflicting
- Bevy handles the flush — no custom `flush_effects` system needed
- Bridge systems between domains stay as regular systems, not exclusive systems

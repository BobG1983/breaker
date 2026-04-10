# Name
Fireable

# Syntax
```rust
trait Fireable {
    fn fire(&self, entity: Entity, source: &str, world: &mut World);
}
```

# Description
The contract for executing an effect on an entity. Every config struct in EffectType implements this trait.

- entity: The entity to apply the effect to (the Owner, or a participant if redirected via On).
- source: The chip or definition name that originated this effect. Used for tracking, removal, and debugging.
- world: Exclusive world access. The implementation can read/write components, spawn entities, send messages — whatever the effect does.

Called by the match dispatch inside FireEffectCommand::apply. Each arm unwraps the config and calls `config.fire(entity, source, world)`.

DO implement Fireable on the config struct, not on EffectType. The enum is the dispatch layer; the config is the implementation.
DO NOT store state between calls. Fire is stateless — each invocation is independent. Persistent state lives in components on the entity.

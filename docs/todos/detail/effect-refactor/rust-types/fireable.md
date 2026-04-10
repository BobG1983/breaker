# Name
Fireable

# Syntax
```rust
trait Fireable {
    fn fire(&self, entity: Entity, source: &str, world: &mut World);

    fn register(app: &mut App) {}
}
```

# Description
The contract for executing an effect on an entity. Every config struct in EffectType implements this trait.

## fire

- entity: The entity to apply the effect to (the Owner, or a participant if redirected via On).
- source: The chip or definition name that originated this effect. Used for tracking, removal, and debugging.
- world: Exclusive world access. The implementation can read/write components, spawn entities, send messages — whatever the effect does.

Called by the match dispatch inside FireEffectCommand::apply. Each arm unwraps the config and calls `config.fire(entity, source, world)`.

DO implement Fireable on the config struct, not on EffectType. The enum is the dispatch layer; the config is the implementation.
DO NOT store state between calls. Fire is stateless — each invocation is independent. Persistent state lives in components on the entity.

## register

- app: The Bevy App, available during plugin build.

Called by EffectPlugin::build for every config struct. The default implementation is a no-op. Override it when the effect has runtime systems, components, or resources to register.

Effects with no runtime infrastructure (passive effects, fire-and-forget effects) use the default no-op. Effects with tick systems, cleanup systems, or reset systems override register to add them to the app.

```rust
impl Fireable for ShockwaveConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) { /* ... */ }

    fn register(app: &mut App) {
        app.add_systems(FixedUpdate, (
            tick_shockwave,
            sync_shockwave_visual,
            apply_shockwave_damage,
            despawn_finished_shockwave,
        ).chain().in_set(EffectSystems::Tick));
    }
}
```

DO register systems, resources, and component hooks here. DO NOT register bridge systems — bridges belong to triggers.

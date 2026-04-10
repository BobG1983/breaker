# Fireable

Every config struct in EffectType implements Fireable. See [rust-types/fireable.md](../../rust-types/fireable.md) for the type definition.

```rust
trait Fireable {
    fn fire(&self, entity: Entity, source: &str, world: &mut World);
    fn register(app: &mut App) {}
}
```

## fire

- `entity`: The entity to apply the effect to.
- `source`: The chip or definition name that caused this effect.
- `world`: Exclusive world access.

Fire is called by the tree walker when it reaches a Fire leaf. The config struct decides what "firing" means:

| Effect category | What fire does | Examples |
|----------------|----------------|----------|
| Passive | Push (source, config) onto the entity's `EffectStack<Self>`. | SpeedBoost, DamageBoost, SizeBoost, BumpForce, QuickStop, Vulnerable, Piercing, RampingDamage |
| Spawner | Spawn one or more child entities in the world. | Shockwave, Explode, SpawnBolts, SpawnPhantom, ChainBolt, MirrorProtocol, ChainLightning, PiercingBeam, TetherBeam, GravityWell |
| Toggle | Insert a marker component on the entity. | FlashStep |
| Protector | Spawn a protective entity (wall, shield). | Shield, SecondWind |
| Message | Send a message (DamageDealt, KillYourself, etc.). | LoseLife, Die, TimePenalty |
| Stateful | Initialize or update internal state on the entity. | CircuitBreaker, EntropyEngine, Anchor |
| Meta | Select from a pool and delegate to the selected effect's fire. | RandomEffect |

### What fire must NOT do

- DO NOT read or write BoundEffects/StagedEffects. That is the tree walker's job.
- DO NOT dispatch triggers. That is the bridge system's job.
- DO NOT evaluate other effects. Fire executes ONE effect. Meta effects (RandomEffect, EntropyEngine) delegate to another effect's fire, but they don't walk trees.
- DO NOT assume which entity you're running on. Fire receives an entity — it could be a bolt, breaker, cell, or wall. Check components if behavior differs by entity type.
- DO NOT panic on missing components. If the entity doesn't have the expected component (e.g. no EffectStack yet), insert it with default.

## register

- `app`: The Bevy App, available during plugin build.

Called by EffectPlugin::build for every config struct. Override when the effect has runtime infrastructure to register. The default is a no-op.

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

### What register should do

- Register tick/update/cleanup systems in the appropriate sets.
- Register reset systems on state transitions (e.g. `OnEnter(NodeState::Running)`).
- Initialize resources the effect needs.

### What register must NOT do

- DO NOT register bridge systems. Bridges belong to triggers.
- DO NOT register shared infrastructure (system sets, SpawnStampRegistry, etc.). That is the plugin's job.
- DO NOT register systems from other effects.

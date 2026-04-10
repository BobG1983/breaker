# Fireable

Every config struct in EffectType implements Fireable. See [rust-types/fireable.md](../../rust-types/fireable.md) for the type definition.

```rust
trait Fireable {
    fn fire(&self, entity: Entity, source: &str, world: &mut World);
}
```

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

## What fire must NOT do

- DO NOT read or write BoundEffects/StagedEffects. That is the tree walker's job.
- DO NOT dispatch triggers. That is the bridge system's job.
- DO NOT evaluate other effects. Fire executes ONE effect. Meta effects (RandomEffect, EntropyEngine) delegate to another effect's fire, but they don't walk trees.
- DO NOT assume which entity you're running on. Fire receives an entity — it could be a bolt, breaker, cell, or wall. Check components if behavior differs by entity type.
- DO NOT panic on missing components. If the entity doesn't have the expected component (e.g. no EffectStack yet), insert it with default.

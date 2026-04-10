# Reversible

Config structs in ReversibleEffectType also implement Reversible. See [rust-types/reversible.md](../../rust-types/reversible.md) for the type definition.

```rust
trait Reversible: Fireable {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World);
}
```

Reverse undoes what fire did. Same parameters. Called by the tree walker when a During condition becomes false or an Until trigger fires.

| Effect category | What reverse does | Examples |
|----------------|-------------------|----------|
| Passive | Remove the matching (source, config) entry from `EffectStack<Self>`. | SpeedBoost, DamageBoost, SizeBoost, BumpForce, QuickStop, Vulnerable, Piercing, RampingDamage |
| Toggle | Remove the marker component. | FlashStep |
| Protector | Despawn the protective entity that fire spawned. | Shield, SecondWind, Pulse |
| Stateful | Reset internal state. | CircuitBreaker, EntropyEngine, Anchor |

## Non-reversible effects

These do NOT implement Reversible. The type system enforces this — they cannot appear as direct children of During/Until.

Shockwave, Explode, SpawnBolts, SpawnPhantom, ChainBolt, MirrorProtocol, ChainLightning, PiercingBeam, TetherBeam, GravityWell, LoseLife, TimePenalty, Die, RandomEffect.

## What reverse must NOT do

- DO NOT read or write BoundEffects/StagedEffects. That is the tree walker's job.
- DO NOT dispatch triggers. Reversal is not an event — it's cleanup.
- DO NOT reverse effects that were never fired. The entity state will be wrong.
- DO NOT assume the entity still has the components fire added. They may have been removed by other effects. Check before acting.
- DO NOT panic on missing components. If the EffectStack is gone, do nothing. If the marker is already removed, do nothing.

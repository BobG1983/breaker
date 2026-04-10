# Name
EffectType

# Syntax
```rust
enum EffectType {
    SpeedBoost(SpeedBoostConfig),
    SizeBoost(SizeBoostConfig),
    DamageBoost(DamageBoostConfig),
    BumpForce(BumpForceConfig),
    QuickStop(QuickStopConfig),
    FlashStep(FlashStepConfig),
    Piercing(PiercingConfig),
    Vulnerable(VulnerableConfig),
    RampingDamage(RampingDamageConfig),
    Attraction(AttractionConfig),
    Anchor(AnchorConfig),
    Pulse(PulseConfig),
    Shield(ShieldConfig),
    SecondWind(SecondWindConfig),
    Shockwave(ShockwaveConfig),
    Explode(ExplodeConfig),
    ChainLightning(ChainLightningConfig),
    PiercingBeam(PiercingBeamConfig),
    SpawnBolts(SpawnBoltsConfig),
    SpawnPhantom(SpawnPhantomConfig),
    ChainBolt(ChainBoltConfig),
    MirrorProtocol(MirrorConfig),
    TetherBeam(TetherBeamConfig),
    GravityWell(GravityWellConfig),
    LoseLife(LoseLifeConfig),
    TimePenalty(TimePenaltyConfig),
    Die(DieConfig),
    CircuitBreaker(CircuitBreakerConfig),
    EntropyEngine(EntropyConfig),
    RandomEffect(RandomEffectConfig),
}
```

# Description
Every variant wraps a config struct. Config structs implement the Fireable trait (and Reversible for the reversible subset). The match dispatch calls `config.fire(entity, source, world)` uniformly for every variant.

See [configs/](../configs/index.md) for all config struct definitions.
See [reversible-effect-type.md](reversible-effect-type.md) for the reversible subset.

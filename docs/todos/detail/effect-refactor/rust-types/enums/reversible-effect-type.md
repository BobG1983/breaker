# Name
ReversibleEffectType

# Syntax
```rust
enum ReversibleEffectType {
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
    CircuitBreaker(CircuitBreakerConfig),
    EntropyEngine(EntropyConfig),
}
```

# Description
The subset of [EffectType](effect-type.md) that can appear as direct Fire children inside During/Until scoped trees. These effects can be cleanly reversed when the scope ends. Config structs in this enum implement both Fireable and Reversible traits.

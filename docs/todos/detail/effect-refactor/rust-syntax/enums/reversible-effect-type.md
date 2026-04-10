# Name
ReversibleEffectType

# Syntax
```rust
enum ReversibleEffectType {
    SpeedBoost(f32),
    SizeBoost(f32),
    DamageBoost(f32),
    BumpForce(f32),
    QuickStop(f32),
    FlashStep,
    Piercing(u32),
    Vulnerable(f32),
    RampingDamage(f32),
    Attraction(AttractionConfig),
    Anchor(AnchorConfig),
    Pulse(PulseConfig),
    Shield(ShieldConfig),
    SecondWind,
    CircuitBreaker(CircuitBreakerConfig),
    EntropyEngine(EntropyConfig),
}
```

# Description
The subset of [EffectType](effect-type.md) that can appear as direct Fire children inside During/Until scoped trees. These effects can be cleanly reversed when the scope ends.

- SpeedBoost: Multiplicative speed scaling. See [speed-boost](../ron-syntax/effects/speed-boost.md)
- SizeBoost: Multiplicative size increase. See [size-boost](../ron-syntax/effects/size-boost.md)
- DamageBoost: Multiplicative damage bonus. See [damage-boost](../ron-syntax/effects/damage-boost.md)
- BumpForce: Multiplicative bump force increase. See [bump-force](../ron-syntax/effects/bump-force.md)
- QuickStop: Breaker deceleration multiplier. See [quick-stop](../ron-syntax/effects/quick-stop.md)
- FlashStep: Enables flash step on breaker. See [flash-step](../ron-syntax/effects/flash-step.md)
- Piercing: Pass through N cells without bouncing. See [piercing](../ron-syntax/effects/piercing.md)
- Vulnerable: Incoming damage multiplier. See [vulnerable](../ron-syntax/effects/vulnerable.md)
- RampingDamage: Flat damage bonus that accumulates per activation. See [ramping-damage](../ron-syntax/effects/ramping-damage.md)
- Attraction: Steer toward nearest entity of a configured type. See [attraction](../ron-syntax/effects/attraction.md), [AttractionConfig](../configs/attraction-config.md)
- Anchor: Plant mechanic for boosted bump force. See [anchor](../ron-syntax/effects/anchor.md), [AnchorConfig](../configs/anchor-config.md)
- Pulse: Periodic shockwave emitter. See [pulse](../ron-syntax/effects/pulse.md), [PulseConfig](../configs/pulse-config.md)
- Shield: Timed visible floor wall. See [shield](../ron-syntax/effects/shield.md), [ShieldConfig](../configs/shield-config.md)
- SecondWind: Invisible one-shot bottom wall. See [second-wind](../ron-syntax/effects/second-wind.md)
- CircuitBreaker: Charge counter that fires reward at threshold. See [circuit-breaker](../ron-syntax/effects/circuit-breaker.md), [CircuitBreakerConfig](../configs/circuit-breaker-config.md)
- EntropyEngine: Escalating chaos with random effects. See [entropy-engine](../ron-syntax/effects/entropy-engine.md), [EntropyConfig](../configs/entropy-config.md)

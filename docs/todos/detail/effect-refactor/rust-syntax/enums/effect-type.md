# Name
EffectType

# Syntax
```rust
enum EffectType {
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
    LoseLife,
    TimePenalty(f32),
    Die,
    CircuitBreaker(CircuitBreakerConfig),
    EntropyEngine(EntropyConfig),
    RandomEffect(Vec<(f32, Box<EffectType>)>),
}
```

# Description
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
- Shockwave: Expanding ring of area damage. See [shockwave](../ron-syntax/effects/shockwave.md), [ShockwaveConfig](../configs/shockwave-config.md)
- Explode: Instant area damage burst. See [explode](../ron-syntax/effects/explode.md), [ExplodeConfig](../configs/explode-config.md)
- ChainLightning: Sequential arc damage jumping between cells. See [chain-lightning](../ron-syntax/effects/chain-lightning.md), [ChainLightningConfig](../configs/chain-lightning-config.md)
- PiercingBeam: Instant damage along bolt velocity direction. See [piercing-beam](../ron-syntax/effects/piercing-beam.md), [PiercingBeamConfig](../configs/piercing-beam-config.md)
- SpawnBolts: Spawn additional bolts. See [spawn-bolts](../ron-syntax/effects/spawn-bolts.md), [SpawnBoltsConfig](../configs/spawn-bolts-config.md)
- SpawnPhantom: Temporary phantom bolt with infinite piercing. See [spawn-phantom](../ron-syntax/effects/spawn-phantom.md), [SpawnPhantomConfig](../configs/spawn-phantom-config.md)
- ChainBolt: Spawn bolt tethered via distance constraint. See [chain-bolt](../ron-syntax/effects/chain-bolt.md), [ChainBoltConfig](../configs/chain-bolt-config.md)
- MirrorProtocol: Spawn mirrored bolt at reflected angle. See [mirror-protocol](../ron-syntax/effects/mirror-protocol.md), [MirrorConfig](../configs/mirror-config.md)
- TetherBeam: Damaging beam between bolts. See [tether-beam](../ron-syntax/effects/tether-beam.md), [TetherBeamConfig](../configs/tether-beam-config.md)
- GravityWell: Pulls bolts within radius toward center. See [gravity-well](../ron-syntax/effects/gravity-well.md), [GravityWellConfig](../configs/gravity-well-config.md)
- LoseLife: Decrement life count. See [lose-life](../ron-syntax/effects/lose-life.md)
- TimePenalty: Subtract seconds from node timer. See [time-penalty](../ron-syntax/effects/time-penalty.md)
- Die: Send entity into death pipeline. See [die](../ron-syntax/effects/die.md)
- CircuitBreaker: Charge counter that fires reward at threshold. See [circuit-breaker](../ron-syntax/effects/circuit-breaker.md), [CircuitBreakerConfig](../configs/circuit-breaker-config.md)
- EntropyEngine: Escalating chaos with random effects. See [entropy-engine](../ron-syntax/effects/entropy-engine.md), [EntropyConfig](../configs/entropy-config.md)
- RandomEffect: Weighted random selection, fires exactly one from pool. See [random-effect](../ron-syntax/effects/random-effect.md)

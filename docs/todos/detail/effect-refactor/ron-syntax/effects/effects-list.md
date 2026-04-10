# Effects

| Effect | RON Syntax | Description |
|--------|-----------|-------------|
| [SpeedBoost](speed-boost.md) | `SpeedBoost(SpeedBoostConfig)` | Multiplicative speed scaling. Stacks multiplicatively. Reversible. |
| [SizeBoost](size-boost.md) | `SizeBoost(SizeBoostConfig)` | Multiplicative size increase. Stacks multiplicatively. Reversible. |
| [DamageBoost](damage-boost.md) | `DamageBoost(DamageBoostConfig)` | Multiplicative damage bonus. Stacks multiplicatively. Reversible. |
| [BumpForce](bump-force.md) | `BumpForce(BumpForceConfig)` | Multiplicative bump force increase. Stacks multiplicatively. Reversible. |
| [QuickStop](quick-stop.md) | `QuickStop(QuickStopConfig)` | Breaker deceleration multiplier. Stacks multiplicatively. Reversible. |
| [FlashStep](flash-step.md) | `FlashStep(FlashStepConfig)` | Enables flash step on breaker. Reversible. |
| [Piercing](piercing.md) | `Piercing(PiercingConfig)` | Pass through N cells without bouncing. Stacks additively. Reversible. |
| [Vulnerable](vulnerable.md) | `Vulnerable(VulnerableConfig)` | Incoming damage multiplier. Stacks multiplicatively. Reversible. |
| [RampingDamage](ramping-damage.md) | `RampingDamage(RampingDamageConfig)` | Flat damage bonus that accumulates per activation. Reversible. |
| [Attraction](attraction.md) | `Attraction(AttractionConfig)` | Steer toward nearest entity of a configured type. Reversible. |
| [Anchor](anchor.md) | `Anchor(AnchorConfig)` | Plant mechanic for boosted bump force. Reversible. |
| [Pulse](pulse.md) | `Pulse(PulseConfig)` | Periodic shockwave emitter. Reversible. |
| [Shield](shield.md) | `Shield(ShieldConfig)` | Timed visible floor wall. Reversible. |
| [SecondWind](second-wind.md) | `SecondWind(SecondWindConfig)` | Invisible one-shot bottom wall. Reversible. |
| [Shockwave](shockwave.md) | `Shockwave(ShockwaveConfig)` | Expanding ring of area damage. Not reversible. |
| [Explode](explode.md) | `Explode(ExplodeConfig)` | Instant area damage burst. Not reversible. |
| [ChainLightning](chain-lightning.md) | `ChainLightning(ChainLightningConfig)` | Sequential arc damage jumping between cells. Not reversible. |
| [PiercingBeam](piercing-beam.md) | `PiercingBeam(PiercingBeamConfig)` | Instant damage along bolt velocity direction. Not reversible. |
| [SpawnBolts](spawn-bolts.md) | `SpawnBolts(SpawnBoltsConfig)` | Spawn additional bolts. Not reversible. |
| [SpawnPhantom](spawn-phantom.md) | `SpawnPhantom(SpawnPhantomConfig)` | Temporary phantom bolt with infinite piercing. Not reversible. |
| [ChainBolt](chain-bolt.md) | `ChainBolt(ChainBoltConfig)` | Spawn bolt tethered via distance constraint. Not reversible. |
| [MirrorProtocol](mirror-protocol.md) | `MirrorProtocol(MirrorConfig)` | Spawn mirrored bolt at reflected angle. Not reversible. |
| [TetherBeam](tether-beam.md) | `TetherBeam(TetherBeamConfig)` | Damaging beam between bolts. Not reversible. |
| [GravityWell](gravity-well.md) | `GravityWell(GravityWellConfig)` | Pulls bolts within radius toward center. Not reversible. |
| [LoseLife](lose-life.md) | `LoseLife(LoseLifeConfig)` | Decrement life count. Not reversible. |
| [TimePenalty](time-penalty.md) | `TimePenalty(TimePenaltyConfig)` | Subtract seconds from node timer. Not reversible. |
| [Die](die.md) | `Die(DieConfig)` | Send entity into death pipeline. Not reversible. |
| [CircuitBreaker](circuit-breaker.md) | `CircuitBreaker(CircuitBreakerConfig)` | Charge counter that fires reward at threshold. Reversible. |
| [EntropyEngine](entropy-engine.md) | `EntropyEngine(EntropyConfig)` | Escalating chaos with random effects. Reversible. |
| [RandomEffect](random-effect.md) | `RandomEffect(RandomEffectConfig)` | Weighted random selection. Not reversible. |

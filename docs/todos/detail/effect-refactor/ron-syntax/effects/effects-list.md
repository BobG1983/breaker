# Effects

| Effect | RON Syntax | Description |
|--------|-----------|-------------|
| [SpeedBoost](speed-boost.md) | `SpeedBoost(f32)` | Multiplicative speed scaling. 1.x standard: 2.0 = double, 0.5 = half. Stacks multiplicatively. Reversible. |
| [SizeBoost](size-boost.md) | `SizeBoost(f32)` | Multiplicative size increase (bolt radius or breaker width). Stacks multiplicatively. Reversible. |
| [DamageBoost](damage-boost.md) | `DamageBoost(f32)` | Multiplicative damage bonus applied when the entity deals damage. Stacks multiplicatively. Reversible. |
| [BumpForce](bump-force.md) | `BumpForce(f32)` | Multiplicative bump force increase. Stacks multiplicatively. Reversible. |
| [QuickStop](quick-stop.md) | `QuickStop(f32)` | Breaker deceleration multiplier. Higher = stops faster. Stacks multiplicatively. Reversible. |
| [FlashStep](flash-step.md) | `FlashStep` | Enables flash step on breaker -- teleport on dash reversal during settling. Reversible. |
| [Piercing](piercing.md) | `Piercing(u32)` | Pass through N cells without bouncing. Stacks additively. Reversible. |
| [Vulnerable](vulnerable.md) | `Vulnerable(f32)` | Incoming damage multiplier on the target entity. Stacks multiplicatively. Reversible. |
| [RampingDamage](ramping-damage.md) | `RampingDamage(f32)` | Flat damage bonus that accumulates per activation. Resets each node. Reversible. |
| [Attraction](attraction.md) | `Attraction(AttractionConfig)` | Steer toward nearest entity of a configured type. Reversible. |
| [Anchor](anchor.md) | `Anchor(AnchorConfig)` | Plant mechanic -- boosted bump force + wider perfect window after standing still. Reversible. |
| [Pulse](pulse.md) | `Pulse(PulseConfig)` | Periodic shockwave emitter attached to entity. Reversible. |
| [Shield](shield.md) | `Shield(ShieldConfig)` | Timed visible floor wall that reflects bolts. Reversible. |
| [SecondWind](second-wind.md) | `SecondWind` | Invisible one-shot bottom wall. Bounces bolt once, consumed. Reversible. |
| [Shockwave](shockwave.md) | `Shockwave(ShockwaveConfig)` | Expanding ring of area damage from entity position. Damage snapshotted at fire time. Not reversible. |
| [Explode](explode.md) | `Explode(ExplodeConfig)` | Instant area damage burst. Flat damage, not modified by boosts. Not reversible. |
| [ChainLightning](chain-lightning.md) | `ChainLightning(ChainLightningConfig)` | Sequential arc damage jumping between cells. Each cell hit at most once per chain. Not reversible. |
| [PiercingBeam](piercing-beam.md) | `PiercingBeam(PiercingBeamConfig)` | Instant damage along bolt's velocity direction within a width. Not reversible. |
| [SpawnBolts](spawn-bolts.md) | `SpawnBolts(SpawnBoltsConfig)` | Spawn additional bolts with random velocity. Optional lifespan and BoundEffects inheritance. Not reversible. |
| [SpawnPhantom](spawn-phantom.md) | `SpawnPhantom(SpawnPhantomConfig)` | Temporary phantom bolt with infinite piercing. Not reversible. |
| [ChainBolt](chain-bolt.md) | `ChainBolt(ChainBoltConfig)` | Spawn bolt tethered to source via distance constraint. Not reversible. |
| [MirrorProtocol](mirror-protocol.md) | `MirrorProtocol(MirrorConfig)` | Spawn mirrored bolt at reflected angle. Not reversible. |
| [TetherBeam](tether-beam.md) | `TetherBeam(TetherBeamConfig)` | Damaging beam between bolts. Not reversible. |
| [GravityWell](gravity-well.md) | `GravityWell(GravityWellConfig)` | Pulls bolts within radius toward center. Self-despawns after duration. Not reversible. |
| [LoseLife](lose-life.md) | `LoseLife` | Decrement life count. RunLost when zero. Not reversible. |
| [TimePenalty](time-penalty.md) | `TimePenalty(f32)` | Subtract seconds from node timer. Not reversible. |
| [Die](die.md) | `Die` | Send entity into death pipeline. Not reversible. |
| [CircuitBreaker](circuit-breaker.md) | `CircuitBreaker(CircuitBreakerConfig)` | Charge counter -- fires reward (spawn + shockwave) at threshold, resets. Reversible. |
| [EntropyEngine](entropy-engine.md) | `EntropyEngine(EntropyConfig)` | Escalating chaos. Fires more random effects as cells die. Counter resets each node. Reversible. |
| [RandomEffect](random-effect.md) | `RandomEffect([(f32, Effect), ...])` | Weighted random selection. Fires exactly one from pool. Not reversible. |

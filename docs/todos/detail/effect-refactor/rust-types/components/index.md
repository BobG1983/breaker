# Effect Components

Component types used by the effect system. Each file documents the Rust struct, post-refactor location, lifecycle (when added, ticked, removed), and owning system.

## Shared Components

These live in `src/effect_v3/components/` and are used across multiple effects.

- [effect-source-chip.md](effect-source-chip.md) -- `EffectSourceChip`: damage attribution to originating chip
- [effect-timers.md](effect-timers.md) -- `EffectTimers`: countdown timers for time-limited effects

## Per-Effect Components

Each effect owns its components in its own module under `src/effect_v3/effects/<name>/`.

- [shockwave.md](shockwave.md) -- `ShockwaveSource`, `ShockwaveRadius`, `ShockwaveMaxRadius`, `ShockwaveSpeed`, `ShockwaveDamaged`, `ShockwaveBaseDamage`, `ShockwaveDamageMultiplier`
- [chain-lightning.md](chain-lightning.md) -- `ChainLightningChain`, `ChainState`
- [anchor.md](anchor.md) -- `AnchorActive`, `AnchorTimer`, `AnchorPlanted`
- [attraction.md](attraction.md) -- `ActiveAttractions`, `AttractionEntry`
- [circuit-breaker.md](circuit-breaker.md) -- `CircuitBreakerCounter`
- [entropy-engine.md](entropy-engine.md) -- `EntropyCounter`
- [flash-step.md](flash-step.md) -- `FlashStepActive`
- [gravity-well.md](gravity-well.md) -- `GravityWellSource`, `GravityWellStrength`, `GravityWellRadius`, `GravityWellLifetime`, `GravityWellOwner`
- [piercing-remaining.md](piercing-remaining.md) -- `PiercingRemaining`
- [pulse-emitter.md](pulse-emitter.md) -- `PulseEmitter`
- [ramping-damage-accumulator.md](ramping-damage-accumulator.md) -- `RampingDamageAccumulator`
- [shield.md](shield.md) -- `ShieldWall`, `ShieldOwner`, `ShieldDuration`, `ShieldReflectionCost`
- [second-wind.md](second-wind.md) -- `SecondWindWall`, `SecondWindOwner`
- [phantom-bolt.md](phantom-bolt.md) -- `PhantomBolt`, `PhantomLifetime`, `PhantomOwner`
- [tether-beam.md](tether-beam.md) -- `TetherBeamSource`, `TetherBeamDamage`

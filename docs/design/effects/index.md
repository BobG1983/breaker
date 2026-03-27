# Effects

Effects are the actions that fire when triggers match. Each effect acts on the entity it lives on.

Any effect can be used passively (bare `Do` at dispatch) or triggered (inside a `When`).

All multipliers use the **1.x standard**: 2.0 = 2x (double), 0.5 = 50% (half).

## Combat Effects

- [Shockwave](shockwave.md) — expanding ring of area damage
- [ChainLightning](chain_lightning.md) — arc damage jumping between cells
- [PiercingBeam](piercing_beam.md) — beam through cells in velocity direction
- [Pulse](pulse.md) — shockwave at every active bolt position
- [Explode](explode.md) — instant area damage burst *(not yet implemented)*

## Bolt Spawning

- [SpawnBolts](spawn_bolts.md) — spawn additional bolts
- [MultiBolt](multi_bolt.md) — spawn multiple bolts with stacking
- [ChainBolt](chain_bolt.md) — spawn a tethered chain bolt
- [SpawnPhantom](spawn_phantom.md) — temporary phantom bolt with infinite piercing

## Stat Modifiers

- [SpeedBoost](speed_boost.md) — multiplicative speed scaling
- [DamageBoost](damage_boost.md) — multiplicative damage bonus
- [Piercing](piercing.md) — pass through cells
- [SizeBoost](size_boost.md) — size increase (bolt radius or breaker width)
- [BumpForce](bump_force.md) — bump force increase
- [ChainHit](chain_hit.md) — chain to additional cells on hit
- [RampingDamage](ramping_damage.md) — stacking damage on cell hits
- [Attraction](attraction.md) — attract toward nearest entity of a type

## Breaker Modifiers

- [TiltControl](tilt_control.md) — tilt sensitivity increase
- [BreakerSpeed](breaker_speed.md) — breaker movement speed increase

## Defensive

- [Shield](shield.md) — temporary breaker protection
- [SecondWind](second_wind.md) — invisible bottom wall, bounces bolt once
- [GravityWell](gravity_well.md) — attracts bolts within radius

## Penalties

- [LoseLife](lose_life.md) — decrements lives
- [TimePenalty](time_penalty.md) — subtracts time from node timer

## Meta

- [RandomEffect](random_effect.md) — weighted random selection from pool
- [EntropyEngine](entropy_engine.md) — counter-gated random effect

## Buff Stacking

| Effect | Stacking | Recalculation |
|--------|----------|---------------|
| SpeedBoost | Multiplicative | `base_speed * product(boosts)`, clamped `[min, max]` |
| DamageBoost | Multiplicative | `base_damage * product(boosts)` |
| Piercing | Additive | `sum(pierce_counts)` |
| SizeBoost | Additive | `base_size + sum(boosts)` |
| BumpForce | Additive | `base_force + sum(boosts)` |

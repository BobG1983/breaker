# Effects

Effects are the actions that fire when triggers match. Each effect acts on the entity it lives on.

Any effect can be used passively (bare `Do` at dispatch) or triggered (inside a `When`).

All multipliers use the **1.x standard**: 2.0 = 2x (double), 0.5 = 50% (half).

## Combat Effects

- [Shockwave](shockwave.md) — expanding ring of area damage
- [ChainLightning](chain_lightning.md) — arc damage jumping between random cells in range
- [PiercingBeam](piercing_beam.md) — fast-expanding beam rectangle in velocity direction
- [Pulse](pulse.md) — timed bolt aura, repeatedly pulses small damage rings
- [Explode](explode.md) — instant area damage burst
- [TetherBeam](tether_beam.md) — damaging beam between two free-moving bolts *(evolution)*

## Bolt Spawning

- [SpawnBolts](spawn_bolts.md) — spawn additional bolts
- [ChainBolt](chain_bolt.md) — spawn two bolts chained together
- [SpawnPhantom](spawn_phantom.md) — temporary phantom bolt with infinite piercing

## Stat Modifiers

- [SpeedBoost](speed_boost.md) — multiplicative speed scaling
- [DamageBoost](damage_boost.md) — multiplicative damage bonus
- [Piercing](piercing.md) — pass through destroyed cells (counted down)
- [SizeBoost](size_boost.md) — multiplicative size increase (varies by entity type)
- [BumpForce](bump_force.md) — multiplicative bump force increase
- [RampingDamage](ramping_damage.md) — stacking damage on any impact
- [Attraction](attraction.md) — attract toward nearest entity of a type

## Breaker Modifiers

- [QuickStop](quick_stop.md) — breaker deceleration multiplier for precise stops

## Defensive

- [Shield](shield.md) — temporary protection (bolt loss immunity on breaker, damage immunity on HP entities)
- [SecondWind](second_wind.md) — invisible bottom wall, bounces bolt once
- [GravityWell](gravity_well.md) — attracts bolts within radius

## Penalties

- [LoseLife](lose_life.md) — decrements lives (reverse restores)
- [TimePenalty](time_penalty.md) — subtracts time from node timer (reverse restores)

## Meta

- [RandomEffect](random_effect.md) — weighted random selection from pool
- [EntropyEngine](entropy_engine.md) — escalating chaos, multiple effects per cell destroyed

## Buff Stacking

| Effect | Stacking | Recalculation |
|--------|----------|---------------|
| SpeedBoost | Multiplicative | `base_speed * product(boosts)`, clamped `[min, max]` |
| DamageBoost | Multiplicative | `base_damage * product(boosts)` |
| Piercing | Additive | `sum(pierce_counts)`, counted down on cell destroy |
| SizeBoost | Multiplicative | `base_size * product(boosts)` (varies by entity type) |
| BumpForce | Multiplicative | `base_force * product(boosts)` |
| QuickStop | Multiplicative | `base_decel * product(boosts)` |

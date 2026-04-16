# Effects — Design Direction

Effects are the actions that fire when triggers match. This index organizes effects by **design category** for chip authoring reference. For the technical reference (config structs, fire/reverse behavior, stacking), see `docs/architecture/effects/effect_reference.md`.

All multipliers use the **1.x standard**: 2.0 = 2x (double), 0.5 = 50% (half).

## Combat Effects

- **Shockwave** — expanding ring of area damage
- **ChainLightning** — arc damage jumping between random cells in range
- **PiercingBeam** — instant damage along velocity direction within a width
- **Pulse** — timed bolt aura, repeatedly pulses small damage rings
- [Explode](explode.md) — instant area damage burst (VFX direction)
- [TetherBeam](tether_beam.md) — damaging beam between bolts *(evolution)* (VFX direction)

## Bolt Spawning

- **SpawnBolts** — spawn additional bolts
- **ChainBolt** — spawn two bolts chained together
- **SpawnPhantom** — temporary phantom bolt with infinite piercing
- [MirrorProtocol](mirror_protocol.md) — spawn mirrored bolts *(evolution)* (VFX direction)

## Stat Modifiers

- **SpeedBoost** — multiplicative speed scaling
- **DamageBoost** — multiplicative damage bonus
- **Piercing** — pass through destroyed cells (counted down)
- **SizeBoost** — multiplicative size increase (varies by entity type)
- **BumpForce** — multiplicative bump force increase
- **RampingDamage** — stacking damage on any impact
- **Attraction** — attract toward nearest entity of a type

## Breaker Modifiers

- [QuickStop](quick_stop.md) — breaker deceleration multiplier (evolution direction)
- [Anchor](anchor.md) — plant mechanic *(evolution)* (VFX direction)
- [FlashStep](flash_step.md) — teleport on dash reversal *(evolution)* (VFX direction)

## Defensive

- **Shield** — temporary floor wall (bolt loss immunity)
- **SecondWind** — invisible bottom wall, bounces bolt once
- **GravityWell** — attracts bolts within radius

## Penalties

- **LoseLife** — decrements lives
- **TimePenalty** — subtracts time from node timer

## Meta

- **RandomEffect** — weighted random selection from pool
- **EntropyEngine** — escalating chaos, multiple effects per cell destroyed
- [CircuitBreaker](circuit_breaker.md) — charge counter *(evolution)* (VFX direction)

## Vulnerable / Die

- **Vulnerable** — multiplicative incoming damage amplification
- **Die** — kill the target entity

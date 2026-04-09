# Per-Effect Behavior Specs

One file per effect. Each file is the **complete source of truth** for implementing that effect in the new system. The implementing agent reads ONLY these specs — not the old `src/effect/` code.

## File Format

Each spec defines:
- **Config struct** — fields, types, defaults, serde attributes
- **Reversible** — yes/no (determines if it can appear in During/Until direct Fire)
- **Fire behavior** — what happens when the effect executes
- **Reverse behavior** — what happens when the effect is undone (if reversible)
- **Components** — what components are inserted/removed/mutated
- **Messages** — what messages are sent
- **Entities** — what entities are spawned (if any)
- **Target** — what entity type this effect applies to (bolt, breaker, cell, etc.)

## Stacking Pattern

Most passive effects use a shared stacking pattern:
- **Component**: `Active<Name>(Vec<T>)` where T is the stacking value type
- **Fire**: push value onto Vec (insert component if absent)
- **Reverse**: find first matching value via epsilon compare, swap_remove
- **Aggregation**: product of all entries (for multipliers) or sum (for counts)
- **Safety**: fire/reverse on despawned entity = no-op, reverse without component = no-op

## Index

### Passive (component insert/remove, reversible)
- [speed_boost.md](speed_boost.md)
- [damage_boost.md](damage_boost.md)
- [size_boost.md](size_boost.md)
- [bump_force.md](bump_force.md)
- [piercing.md](piercing.md)
- [quick_stop.md](quick_stop.md)
- [flash_step.md](flash_step.md)
- [vulnerable.md](vulnerable.md)
- [ramping_damage.md](ramping_damage.md)
- [anchor.md](anchor.md)
- [attraction.md](attraction.md)

### Damage-dealing (spawn entities, NOT reversible)
- [shockwave.md](shockwave.md)
- [explode.md](explode.md)
- [chain_lightning.md](chain_lightning.md)
- [piercing_beam.md](piercing_beam.md)
- [pulse.md](pulse.md)
- [tether_beam.md](tether_beam.md)

### Spawn (spawn entities, NOT reversible)
- [spawn_bolts.md](spawn_bolts.md)
- [spawn_phantom.md](spawn_phantom.md)
- [chain_bolt.md](chain_bolt.md)
- [mirror_protocol.md](mirror_protocol.md)

### Utility (special behavior)
- [shield.md](shield.md)
- [second_wind.md](second_wind.md)
- [gravity_well.md](gravity_well.md)
- [circuit_breaker.md](circuit_breaker.md)
- [entropy_engine.md](entropy_engine.md)
- [random_effect.md](random_effect.md)

### Terminal (no config, simple action)
- [lose_life.md](lose_life.md)
- [time_penalty.md](time_penalty.md)
- [die.md](die.md)

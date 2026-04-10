# Name
PiercingBeam

# Enum Variant
- `EffectType::PiercingBeam(PiercingBeamConfig)`

# Config
`PiercingBeamConfig { damage_mult: f32, width: f32 }`

# Fire
1. Read the source entity's position and velocity direction.
2. Calculate the beam rectangle: origin at source position, extending along the velocity direction, with `config.width` perpendicular extent.
3. Query the quadtree for all cells within the beam rectangle.
4. For each cell found, send `DamageDealt<Cell>` with `damage_mult * BoltBaseDamage * EffectStack<DamageBoostConfig>.aggregate()`.
5. Spawn a visual beam entity at the source position along the velocity direction (short-lived VFX, not persistent).
6. Fire does NOT spawn a persistent entity -- beam is instant damage plus a short VFX.

# Reverse
Not reversible.

# Source Location
`src/effect/effects/piercing_beam/config.rs`

# New Types
None persistent. The visual beam is a short-lived VFX entity using existing VFX infrastructure.

# New Systems
None -- fully resolved in fire.

# MirrorProtocol

Spawn a mirrored bolt from any bolt on impact. The mirror position and velocity depend on which side of the collider was hit — creating a symmetric "reflection" that goes the opposite direction on the appropriate axis.

For technical details (config struct, mirror computation, fire behavior), see `docs/architecture/effects/effect_reference.md`.

## Ingredients

Reflex x1 + Piercing Shot x2.

## VFX

- On trigger: brief prismatic flash at the bolt's impact point
- Mirrored bolt emerges from the flash with prismatic birth trail
- The mirror direction (horizontal or vertical) is visually readable from the flash orientation

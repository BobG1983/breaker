# Name
MirrorProtocol

# Enum Variant
- `EffectType::Mirror(MirrorConfig)`

# Config
`MirrorConfig { inherit: bool }`

# Fire
1. Read the source bolt entity's position and velocity.
2. Calculate the mirrored velocity by reflecting across the vertical axis (negate the x component).
3. Spawn a new bolt entity at the source position with the mirrored velocity.
4. Mark the spawned bolt as `ExtraBolt`.
5. If `inherit` is `true`, clone the source bolt's `BoundEffects` onto the mirrored bolt. Cloned entries keep their original source strings — the clone shares fate with the original (if the chip is unequipped, both lose their effects).
6. Fire does NOT manage the mirrored bolt -- it is a regular bolt after spawn.

# Reverse
Not reversible.

# Source Location
`src/effect/effects/mirror_protocol/config.rs`

# New Types
None -- uses existing bolt builder and existing `ExtraBolt` marker.

# New Systems
None -- fully resolved in fire.

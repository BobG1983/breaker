# Name
Explode

# Enum Variant
- `EffectType::Explode(ExplodeConfig)`

# Config
`ExplodeConfig { range: f32, damage: f32 }`

# Fire
1. Read the source entity's position.
2. Query the quadtree for all cells within `range` of the source position.
3. For each cell found, send `DamageDealt<Cell>` with `config.damage` as flat damage.
4. Spawn a visual flash entity at the source position (short-lived VFX, not persistent).
5. Fire does NOT spawn a persistent entity -- explosion is instant.
6. Fire does NOT use damage boosts -- damage is flat from config.

# Reverse
Not reversible.

# Source Location
`src/effect_v3/effects/explode/config.rs`

# New Types
None persistent. The visual flash is a short-lived VFX entity using existing VFX infrastructure.

# New Systems
None -- explosion is fully resolved in fire.

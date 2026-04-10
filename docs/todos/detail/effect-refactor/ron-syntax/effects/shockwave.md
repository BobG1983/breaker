# Name
Shockwave

# Parameters
`ShockwaveConfig`

# Description
Spawns an expanding ring of area damage centered on the entity's position. The ring starts small and grows outward at a configured speed. Every cell the ring passes through takes damage exactly once. The damage amount is snapshotted from the source entity's base damage and active damage boosts at the moment the shockwave is created -- later changes to damage boosts don't affect an in-flight shockwave. See [ShockwaveConfig](../configs/shockwave-config.md).

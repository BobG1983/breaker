# Name
Fire

# Parameters
- Effect: The effect to execute

# Description
Fire is the leaf node that actually does something. It executes an effect immediately on the entity that owns the effect tree. Fire(SpeedBoost(1.5)) makes the entity faster right now. Fire(Shockwave(...)) spawns a shockwave from the entity's position right now.

Fire is a terminal — nothing nests inside it. It's always the end of a chain. When(..., Fire(...)), Once(..., Fire(...)), During(..., Fire(...)) — Fire is always the last step.

Fire always targets the Owner — the entity whose effect tree is being evaluated. To fire an effect on a different entity (like a trigger participant), use On() to redirect first.

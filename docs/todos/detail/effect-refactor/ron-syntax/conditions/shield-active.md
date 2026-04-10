# Name
ShieldActive

# Parameters
None

# Description
True while at least one ShieldWall entity exists in the world. Becomes true when the first shield spawns, becomes false when the last shield despawns.

During(ShieldActive, Fire(DamageBoost(2.0))) means "double damage while a shield is protecting you." The boost activates when any shield appears and deactivates when all shields are gone. If a new shield spawns after the last one expired, the boost reactivates.

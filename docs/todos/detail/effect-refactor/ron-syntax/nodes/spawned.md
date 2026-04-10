# Name
Spawned

# Parameters
- EntityKind: What type of entity to watch for
- Tree: The effect tree to apply to newly spawned entities

# Description
Spawned fires its inner tree whenever a new entity of the specified kind appears in the world. Spawned(Bolt, Fire(SpeedBoost(1.5))) means "every bolt that spawns gets a speed boost."

This is how EveryBolt-style stamp targets work behind the scenes — Stamp(EveryBolt, ...) stamps onto existing bolts and registers a Spawned(Bolt, ...) entry so future bolts get the same tree.

Spawned(Any, ...) watches for all entity types — bolts, cells, walls, breakers.

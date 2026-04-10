# Name
Spawn

# Parameters
- EntityKind: What type of entity to watch for
- Tree: The effect tree to apply to newly spawned entities

# Description
Spawn is a root node — it appears at the top level of an `effects: []` list. It watches for new entities of the specified kind and applies the inner tree to each one as it appears.

Spawn(Bolt, Fire(SpeedBoost(1.5))) means "every bolt that spawns gets a speed boost."

Spawn(Cell, Fire(Vulnerable(2.0))) means "every cell that spawns takes double damage."

Spawn(Any, Fire(SpeedBoost(1.0))) watches for all entity types.

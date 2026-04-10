# Name
Stamp

# Parameters
- StampTarget: Which entity or entities receive the effect tree
- Tree: The effect tree to apply

# Description
Stamp is the top-level wrapper for every effect definition. It declares which entity type the effect tree should be applied to. Every entry in an `effects: []` list must begin with Stamp — you cannot have a bare When or Fire at the top level.

When a chip, breaker, or cell definition is loaded, each Stamp entry is resolved: the StampTarget is matched to actual game entities, and the inner tree is permanently installed on those entities. The tree stays on the entity for as long as the source (chip, breaker definition, etc.) is active.

Stamp also appears as a terminal inside On() — in that context it permanently installs a tree onto another entity at runtime. For example, "when I bump perfectly, permanently give the breaker a speed boost" would use On(BumpTarget::Breaker, Stamp(Fire(SpeedBoost(1.5)))). The stamped tree persists and re-arms just like a definition-level Stamp.

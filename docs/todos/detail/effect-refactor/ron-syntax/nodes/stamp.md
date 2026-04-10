# Name
Stamp

# Parameters
- StampTarget: Which entity or entities receive the effect tree
- Tree: The effect tree to apply

# Description
Stamp is a root node — it appears at the top level of an `effects: []` list. It declares which entity type the effect tree should be applied to.

When a chip, breaker, or cell definition is loaded, each Stamp entry is resolved: the StampTarget is matched to actual game entities, and the inner tree is installed on those entities. The tree stays on the entity for as long as the source (chip, breaker definition, etc.) is active.

Stamp(Bolt, When(Impacted(Wall), Fire(Shockwave(ShockwaveConfig(...))))) means "on the bolt, every time it hits a wall, fire a shockwave."

Stamp(Breaker, During(NodeActive, Fire(SpeedBoost(1.5)))) means "on the breaker, while the node is active, apply a speed boost."

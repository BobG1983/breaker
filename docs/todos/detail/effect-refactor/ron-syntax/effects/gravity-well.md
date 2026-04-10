# Name
GravityWell

# Parameters
`GravityWellConfig`

# Description
Spawns a gravity well at the entity's position that pulls all bolts within its radius toward its center. The well lasts for a configured duration then disappears. Bolts inside the radius have their direction bent toward the well's center without changing speed. A maximum cap limits how many wells a single source can have active -- if exceeded, the oldest well is removed. See [GravityWellConfig](../configs/gravity-well-config.md).

# Name
DamageBoost

# Parameters
`DamageBoostConfig` — See [DamageBoostConfig](../configs/damage-boost-config.md)

# Description
Increases the damage the entity deals. When the bolt hits a cell, the base damage is multiplied by the product of all active damage boosts. A bolt with DamageBoost(DamageBoostConfig(multiplier: 2.0)) deals double damage on every hit. Stacks multiplicatively.

# Name
SpeedBoost

# Parameters
`SpeedBoostConfig` — See [SpeedBoostConfig](../configs/speed-boost-config.md)

# Description
Speeds up or slows down the entity. A bolt with SpeedBoost(SpeedBoostConfig(multiplier: 2.0)) moves at twice its base speed. Multiple speed boosts stack multiplicatively -- two multiplier: 1.5 entries result in 2.25x speed. The final speed is clamped between the bolt's minimum and maximum speed limits.

# Name
SpeedBoost

# Parameters
`f32` -- multiplier

# Description
Speeds up or slows down the entity. A bolt with SpeedBoost(2.0) moves at twice its base speed. Multiple speed boosts stack multiplicatively -- two SpeedBoost(1.5) entries result in 2.25x speed. The final speed is clamped between the bolt's minimum and maximum speed limits.

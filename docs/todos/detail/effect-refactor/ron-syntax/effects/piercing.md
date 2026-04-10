# Name
Piercing

# Parameters
`u32` -- count

# Description
Allows the bolt to pass through cells without bouncing off them. Each cell the bolt passes through decrements the piercing counter by one. When the counter reaches zero, the bolt bounces normally on the next hit. Multiple piercing effects stack additively -- Piercing(2) + Piercing(1) = 3 cells before bouncing.

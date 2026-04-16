# QuickStop

Breaker deceleration multiplier — enables precise stops at high speed.

For technical details (config struct, stacking, fire/reverse behavior), see `docs/architecture/effects/effect_reference.md`.

## Evolution: FlashStep

On a successful quick stop, the breaker is immediately teleported to the X position directly under the lowest active bolt's Y position. Enables "teleport-and-plant" playstyle at high stacks.

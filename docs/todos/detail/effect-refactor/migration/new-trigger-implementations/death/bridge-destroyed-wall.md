# Name
bridge_destroyed::\<Wall\>

# SystemSet
`EffectSystems::Bridge`. Runs in FixedUpdate.

# Filepath
`src/effect/bridges/destroyed.rs` — lives in the effect domain because it calls `walk_effects`. Registered by EffectPlugin, not the death pipeline plugin. — same generic system, monomorphized for Wall.

# Queries/Filters
No queries — reads `Destroyed<Wall>` messages only.

# Description
Same behavior as bridge_destroyed::\<Cell\> but for wall deaths.

1. Dispatch `Died` on victim (the wall).
2. If killer exists, dispatch `Killed(Wall)` on the killer.
3. Dispatch `DeathOccurred(Wall)` globally.

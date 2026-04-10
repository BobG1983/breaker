# Name
bridge_destroyed::\<Bolt\>

# SystemSet
`DeathPipelineSystems::BridgeDestroyed`. Runs in FixedUpdate.

# Filepath
`src/shared/systems/bridge_destroyed.rs` — same generic system, monomorphized for Bolt.

# Queries/Filters
No queries — reads `Destroyed<Bolt>` messages only.

# Description
Same behavior as bridge_destroyed::\<Cell\> but for bolt deaths.

1. Dispatch `Died` on victim (the bolt).
2. If killer exists, dispatch `Killed(Bolt)` on the killer.
3. Dispatch `DeathOccurred(Bolt)` globally.

Most bolt deaths are environmental (killer = None), so step 2 is usually skipped.

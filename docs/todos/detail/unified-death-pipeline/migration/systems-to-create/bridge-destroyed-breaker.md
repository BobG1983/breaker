# Name
bridge_destroyed::\<Breaker\>

# SystemSet
`DeathPipelineSystems::BridgeDestroyed`. Runs in FixedUpdate.

# Filepath
`src/shared/systems/bridge_destroyed.rs` — same generic system, monomorphized for Breaker.

# Queries/Filters
No queries — reads `Destroyed<Breaker>` messages only.

# Description
Same behavior as bridge_destroyed::\<Cell\> but for breaker deaths.

1. Dispatch `Died` on victim (the breaker).
2. If killer exists, dispatch `Killed(Breaker)` on the killer.
3. Dispatch `DeathOccurred(Breaker)` globally.

Breaker deaths are typically environmental (all lives lost from bolt loss), so killer is usually None and step 2 is skipped.

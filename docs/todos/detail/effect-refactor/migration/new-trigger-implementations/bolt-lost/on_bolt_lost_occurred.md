# Name
on_bolt_lost_occurred

# Reads
`BoltLost` message.

# Dispatches
`Trigger::BoltLostOccurred`

# Scope
Global (exception: carries participants).

Walk ALL entities with `BoltLostOccurred` trigger attached.

# TriggerContext
`TriggerContext::BoltLost { bolt: Entity, breaker: Entity }`

- `bolt` -- the bolt entity that was lost.
- `breaker` -- the breaker entity that lost it. May need to be queried at dispatch time since `BoltLost` is currently a unit struct and does not carry entity references.

# Source Location
`src/effect/bridges/bolt_lost.rs`

# Schedule
FixedUpdate, after `BoltSystems::BoltLost`.

# Behavior
1. Read each `BoltLost` message.
2. Determine which bolt was lost and which breaker lost it (query if not in message).
3. Walk ALL entities that have `BoltLostOccurred` in their trigger set.
4. For each match, invoke the tree walker with `TriggerContext::BoltLost { bolt, breaker }`.

- Does NOT despawn the bolt -- fire the trigger before despawn.
- Does NOT decrement lives -- that is the `LoseLife` effect's job via the tree walker.
- Does NOT modify any bolt or breaker state.

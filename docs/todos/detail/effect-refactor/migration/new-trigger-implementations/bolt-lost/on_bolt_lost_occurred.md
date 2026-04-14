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

- `bolt` -- the bolt entity that was lost (from `msg.bolt`).
- `breaker` -- the breaker entity that lost it (from `msg.breaker`).

# Source Location
`src/effect_v3/triggers/bolt_lost/bridges.rs`

# Schedule
FixedUpdate, after `BoltSystems::BoltLost`.

# Behavior
1. Read each `BoltLost` message.
2. Read `bolt` and `breaker` from the message.
3. Walk ALL entities that have `BoltLostOccurred` in their trigger set.
4. For each match, invoke the tree walker with `TriggerContext::BoltLost { bolt, breaker }`.

- Does NOT despawn the bolt -- fire the trigger before despawn.
- Does NOT decrement lives -- that is the `LoseLife` effect's job via the tree walker.
- Does NOT modify any bolt or breaker state.

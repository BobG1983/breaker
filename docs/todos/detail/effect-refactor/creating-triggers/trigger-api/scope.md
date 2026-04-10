# Trigger Scope

Every trigger has a scope that determines which entities the walker visits.

## Local

The trigger fires only on the participant entities involved in the event.

Use Local when the trigger is about a specific interaction between entities — a bump between a bolt and a breaker, a collision between a bolt and a cell, a death where a killer killed a victim.

The walker runs on each participant entity. TriggerContext carries the participants so On() nodes can resolve to specific entities.

Examples: PerfectBumped, EarlyBumped, LateBumped, Bumped, Impacted(EntityKind), Died, Killed(EntityKind).

## Global

The trigger fires on every entity in the world that has BoundEffects or StagedEffects.

Use Global when the trigger is a world-level event that any entity might want to react to — a node started, a cell died somewhere, a bump happened somewhere. The entities being walked are NOT necessarily involved in the event.

TriggerContext is None for most global triggers — On() nodes have no participant to resolve. Exception: BoltLostOccurred is global but carries bolt and breaker participants because both need to react and the specific entities are known.

Examples: PerfectBumpOccurred, BumpOccurred, ImpactOccurred(EntityKind), DeathOccurred(EntityKind), BoltLostOccurred, NodeStartOccurred, NodeEndOccurred, NodeTimerThresholdOccurred.

## Self

The trigger fires only on the single entity that owns the timer or state.

Use Self when the trigger is internal to one entity — a countdown expired, an internal threshold crossed. No other entity is involved.

TriggerContext is None. On() nodes have no participant to resolve.

Examples: TimeExpires.

## Choosing a scope

- Does the trigger involve two entities interacting? → Local.
- Does the trigger announce something happened that anyone might care about? → Global.
- Does the trigger come from inside one entity with no external event? → Self.
- Does the trigger need both Local and Global? Many game events produce a Local trigger on participants AND a Global trigger on everyone. The bridge dispatches both.

# Dispatch Algorithm

A game event happens. It becomes a Trigger. The trigger's scope determines which entities are notified. For each notified entity, run the walking algorithm.

## Steps

1. A game event occurs (bump, impact, death, bolt lost, node lifecycle, timer expiry).
2. The event is translated into one or more Triggers with a trigger context (which entities were involved).
3. Determine scope:
   - **Local** — walk only the participant entities involved in the event.
   - **Global** — walk every entity in the world that has BoundEffects or StagedEffects.
   - **Self** — walk only the owner entity that set up the trigger.
4. For each entity in scope, run the walking algorithm (see walking-effects/walking-algorithm.md) with the trigger and trigger context.

## Trigger Context

The trigger context carries the entities involved in the event so that On nodes can resolve ParticipantTargets during tree walking. **All triggers populate their context** — including global triggers. On() nodes can resolve participants even inside global trigger trees.

### Context Population Table

| Trigger | Scope | TriggerContext |
|---------|-------|---------------|
| PerfectBumped | Local | `Bump { bolt, breaker }` |
| EarlyBumped | Local | `Bump { bolt, breaker }` |
| LateBumped | Local | `Bump { bolt, breaker }` |
| Bumped | Local | `Bump { bolt, breaker }` |
| PerfectBumpOccurred | Global | `Bump { bolt, breaker }` |
| EarlyBumpOccurred | Global | `Bump { bolt, breaker }` |
| LateBumpOccurred | Global | `Bump { bolt, breaker }` |
| BumpOccurred | Global | `Bump { bolt, breaker }` |
| BumpWhiffOccurred | Global | `None` (no participants in a whiff) |
| NoBumpOccurred | Global | `Bump { bolt: msg.bolt, breaker }` |
| Impacted(EntityKind) | Local | `Impact { impactor, impactee }` |
| ImpactOccurred(EntityKind) | Global | `Impact { impactor, impactee }` |
| Died | Local | `Death { victim, killer }` |
| Killed(EntityKind) | Local | `Death { victim, killer }` |
| DeathOccurred(EntityKind) | Global | `Death { victim, killer }` |
| BoltLostOccurred | Global | `BoltLost { bolt, breaker }` |
| NodeStartOccurred | Global | `None` |
| NodeEndOccurred | Global | `None` |
| NodeTimerThresholdOccurred(f32) | Global | `None` |
| TimeExpires(f32) | Self | `None` |

## Multiple Triggers from One Event

A single game event may produce multiple triggers. A perfect bump produces both PerfectBumped (Local) and PerfectBumpOccurred (Global). A cell death produces Died (Local, on victim), Killed(Cell) (Local, on killer), and DeathOccurred(Cell) (Global). Each trigger is dispatched independently.

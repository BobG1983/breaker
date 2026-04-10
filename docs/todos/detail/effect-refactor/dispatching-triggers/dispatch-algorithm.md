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

The trigger context carries the entities involved in the event so that On nodes can resolve ParticipantTargets during tree walking. Each trigger category populates different context fields — see the per-trigger files for specifics.

## Multiple Triggers from One Event

A single game event may produce multiple triggers. A perfect bump produces both PerfectBumped (Local) and PerfectBumpOccurred (Global). A cell death produces Died (Local, on victim), Killed(Cell) (Local, on killer), and DeathOccurred(Cell) (Global). Each trigger is dispatched independently.

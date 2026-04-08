# AnimateIn / AnimateOut Timing

## Decision

`AnimateIn` and `AnimateOut` state phases **must be snappy**. These phases exist for visual polish (bolt birthing, cell animations, transition effects), but they gate gameplay — the player cannot interact until they complete.

Any animation that runs during `AnimateIn` or `AnimateOut` must complete in **under 200ms**. Longer durations create dead air, violating the speed pillar.

## Rationale

The game's identity is relentless forward momentum (see [pillars/1-escalation.md](../pillars/1-escalation.md)). Between nodes, the player already waits through:
1. Chip select screen
2. Fade-out transition
3. AnimateIn phase (birthing, cell spawn animations)
4. Serving state (waiting for launch input)

Each phase adds sequential latency. AnimateIn is the most dangerous because it's mandatory — the player can't skip it. If animations take 300ms+, the cumulative inter-node downtime becomes noticeable and breaks the pacing.

## Constraints

- `BIRTHING_DURATION` (bolt scale-up): **150ms** with ease-out curve
- Future cell animations: **must stay under 200ms**
- `AnimateOut` (if added): same constraint
- These durations are compile-time constants, not RON config — they're feel parameters, not gameplay parameters

## Litmus Test

> "Does this feel fast?" If it doesn't, speed it up until it does.

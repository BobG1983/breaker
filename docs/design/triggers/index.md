# Triggers

Triggers are conditions that gate effect evaluation. Each trigger has a **scope** (global or targeted).

## Bump Triggers

- [PerfectBump](perfect_bump.md) — global — perfect-timed bump happened
- [PerfectBumped](perfect_bumped.md) — targeted (bolt) — "I was perfect bumped"
- [EarlyBump](early_bump.md) — global — early bump happened
- [EarlyBumped](early_bumped.md) — targeted (bolt) — "I was early bumped"
- [LateBump](late_bump.md) — global — late bump happened
- [LateBumped](late_bumped.md) — targeted (bolt) — "I was late bumped"
- [Bump](bump.md) — global — any non-whiff bump happened
- [Bumped](bumped.md) — targeted (bolt) — "I was bumped" (any non-whiff)
- [BumpWhiff](bump_whiff.md) — global — bump window expired without contact
- [NoBump](no_bump.md) — global — bolt passed breaker without bump attempt

## Impact Triggers

- [Impact](impact.md) — global — "there was an impact involving an X"
- [Impacted](impacted.md) — targeted (both participants) — "you were in an impact with X"

## Death Triggers

- [Death](death.md) — global — something died
- [Died](died.md) — targeted — "I died"

## Destruction Triggers

- [BoltLost](bolt_lost.md) — global — bolt fell off screen
- [CellDestroyed](cell_destroyed.md) — global — a cell was destroyed

## Node Lifecycle Triggers

- [NodeStart](node_start.md) — global — a new node has started
- [NodeEnd](node_end.md) — global — the current node has ended

## Timer Triggers

- [NodeTimerThreshold](node_timer_threshold.md) — global — node timer ratio drops below threshold
- [TimeExpires](time_expires.md) — special — timer-based removal for Until nodes

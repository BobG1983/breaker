---
name: Phase 2b Run Structure & Timer Review
description: Design evaluation of 3-node run, node timer, timer UI thresholds, no grace period, binary outcome, and run-end screen
type: project
---

## Phase 2b Evaluation (2026-03-13, updated)

### Approved
- 3-node sequential run: correct scope for Phase 2b
- Node timer countdown-to-loss: core tension mechanic, clean implementation
- No grace period on timer expiry: do not soften
- Binary win/lost: correct placeholder before meta-progression exists
- Timer UI thresholds at 33%/15%: IMPLEMENTED, correct values
- Escalation HP/s ratios: Scatter 0.57, Corridor 0.72, Fortress 1.0 -- CORRECT
- System ordering: track_node_completion before tick_node_timer -- player cannot lose on clear frame
- NodeTransition is 1-frame transient state -- clean, minimal dead air

### Remaining Issues (not blocking, should fix before design-complete)
- **"The clock ran out." subtitle**: Too passive, externalizes failure. Replace with "Node X/3" or node name, or cut entirely.
- **Run-end screen dead air**: Static "Press Enter to continue" with no timer. Add 3s auto-advance (skippable with Enter).
- **"All nodes cleared!" subtitle**: Tells player what they already know. Cut it or replace with time stats.

### Parameter Values (current)
- Scatter: 60s timer, 22 cells (6T+16S), 34 HP, 0.57 HP/s required
- Corridor: 75s timer, 30 cells (12T+18S), 54 HP, 0.72 HP/s required
- Fortress: 70s timer, 50 cells (10T+40S), 70 HP, 1.00 HP/s required

### Future Hooks Verified
- Chrono archetype: NodeTimer is mutable resource, bolt-lost time dock needs no arch changes
- Amps/speed upgrades: faster clears = more time margin, timer creates value prop
- Phase 4 visuals: fraction calculation in color_for_fraction is right abstraction for screen effects
- Phase 7 meta: RunState tracks node_index + outcome, time-remaining stats are clean extension

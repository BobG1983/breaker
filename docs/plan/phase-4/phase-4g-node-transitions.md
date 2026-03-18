# Phase 4g: Node Transitions & VFX

**Goal**: Animated transitions between nodes — on clear and on load. Multiple transition styles, randomly selected per seed.

**Wave**: 3 (integration) — parallel with 4f. **Session 7.**

## Dependencies

- 4e (Node Escalation) — need something to transition between

## What to Build

### Transition State Machine

Replace the current 1-frame `NodeTransition` state with a timed transition:

```
NodeCleared -> TransitionOut (animated wipe) -> ChipSelect -> TransitionIn (animated load) -> Playing
```

Or if no chip select (e.g., boss drops evolution instead):

```
NodeCleared -> TransitionOut -> EvolutionReward -> TransitionIn -> Playing
```

### Transition Styles

Define multiple transition effects, all matching the neon cyberpunk aesthetic:

Examples (specific VFX designed during implementation):
- **Shatter**: Cells explode outward in neon particles, screen clears from center
- **Sweep**: Neon horizontal wipe, old scene slides out as new scene slides in
- **Dissolve**: Cells dissolve into pixel dust, new cells materialize from dust
- **Flash**: Quick white flash with chromatic aberration, scene swaps

Each transition has:
- A "clear" variant (on node clear — celebratory, fast)
- A "load" variant (on node load — build-up, revealing)
- Duration in seconds (short — 0.5-1.0s to maintain pace)

### Random Selection

- Transition style selected from pool using seeded RNG
- Same seed = same transitions throughout the run
- Can be overridden per scenario for testing

### Timing

Transitions must be **quick** to maintain the game's pace. Target:
- Clear transition: 0.5-0.8 seconds
- Load transition: 0.3-0.5 seconds
- Total node-to-node gap (including chip select): should feel snappy, not sluggish

## Acceptance Criteria

1. Node clear plays a visible animated transition
2. Node load plays a visible animated transition
3. Multiple transition styles exist and are visually distinct
4. Transitions match the neon cyberpunk aesthetic
5. Total transition time is under 1 second per direction
6. Same seed produces same transition sequence

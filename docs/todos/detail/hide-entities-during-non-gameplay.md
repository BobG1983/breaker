# Hide Breaker/Bolt During Non-Gameplay States

## Summary
Breaker and bolt should despawn (or at minimum be hidden) when entering RunEnd/ChipSelect states — currently they remain visible during non-gameplay states.

## Context
During a play session (2026-04-06), breaker and bolt entities remain rendered on screen when the game transitions from NodeState::Playing to RunEnd or ChipSelect. They should either be despawned with CleanupOnNodeExit or have their visibility toggled off. The breaker persists across nodes (CleanupOnRunEnd), but it shouldn't be visible during non-gameplay screens.

## Scope
- In: Hide or despawn breaker and bolt entities during RunEnd, ChipSelect, and any other non-gameplay RunState
- In: Ensure they reappear correctly when the next node starts
- Out: Menu state handling (breaker/bolt don't exist yet during menus)

## Dependencies
- Depends on: state lifecycle (already refactored)
- Related to: centralized despawn system (if implemented first, could use that)

## Notes
- Option A: Toggle `Visibility` component on state transitions (simpler, entities persist)
- Option B: Despawn on node exit, re-create on node enter (cleaner but more work)
- Option A is probably better since the breaker already persists across nodes via CleanupOnRunEnd

## Status
`ready`

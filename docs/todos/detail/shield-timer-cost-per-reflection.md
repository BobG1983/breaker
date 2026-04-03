# Shield Timer Cost Per Reflection

## Summary
Each bolt reflection off the Shield wall should cost ~0.5-1.0 seconds off the timer, making the shield degrade under pressure.

## Context
The guard-game-design agent flagged this during the wall builder feature review. The Shield wall currently has a flat 5-second duration regardless of how many bolts it saves. Adding a per-reflection timer cost creates the right tension profile: a player who needed 4 saves gets maybe 1 second of residual protection, while a player who positioned well and needed 0 saves gets the full window for aggression.

## Scope
- In: Listen for `BoltImpactWall` messages where the wall has `ShieldWall` marker. On each impact, subtract a configurable cost (e.g., 0.75s) from `ShieldWallTimer`.
- In: Make the cost configurable in RON (new field on `EffectKind::Shield` or in `WallDefinition`)
- In: Visual decay — wall flickers, dims, or narrows as timer runs down (ties into Phase 5j)
- Out: Partial-width walls (Phase 7 enrichment)

## Dependencies
- Depends on: Shield wall refactor (done — wall builder pattern feature)
- Depends on: `BoltImpactWall` message (exists)

## Notes
- The system would run after `BoltSystems::WallCollision` (same as `tick_shield_wall_timer`), reading `BoltImpactWall` messages and decrementing the timer on matching `ShieldWall` entities.
- Also consider tuning base duration from 5.0 to 3.0 (game design feedback: 5s is too generous).

## Status
`[NEEDS DETAIL]`

# Phase 1 Core Mechanics Review (2026-03-10)

## Breaker State Machine: PASS
- Idle -> Dashing -> Braking -> Settling -> Idle matches design doc
- Dash requires directional commitment (velocity or held key)
- Tilt flows naturally through states: dash forward, brake opposite, settle neutral

## Bolt Reflection: PASS
- Direction fully overwritten on breaker contact -- no incoming angle carryover
- hit_fraction + tilt angle + speed preservation all correct
- speed.max(base_speed) prevents momentum death spirals

## Bump System: PASS with concerns
- Timing grades correct: Early/Late punish (0.8x), Perfect rewards (1.5x), None neutral (1.0x)
- CONCERN: apply_bump_grade uses bolt_velocity.y > 0 as collision proxy -- fragile
- CONCERN: Perfect bump dash-cancel deferred -- must be implemented (crown jewel mechanic)

## Bolt-Cell Collision: PASS
- AABB + face-based reflection + health decrement correct
- "One cell per tick" limit fine for Phase 1, revisit for piercing Amps

## Bolt-Lost: CONDITIONAL PASS
- Respawn works but is too forgiving (base speed, no penalty feel)
- Recommendation: respawn at min_speed to create momentum-rebuild pressure

## Min Angle Enforcement: PASS
- ~10deg from horizontal prevents tedious lateral ping-pong
- Preserves speed and sign when correcting

## Parameter Assessment
- Perfect window (50ms / ~3 frames at 60fps) is tight but learnable -- fighting game parry territory
- Bump anticipation distance ~60 world units at base speed -- react to bolt approach
- Brake tilt > dash tilt creates interesting "which angle do I want" decisions
- Dash covers ~half playfield width -- emergency repositioning, not safe travel

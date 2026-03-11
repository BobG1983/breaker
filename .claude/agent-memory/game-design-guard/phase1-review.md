# Phase 1 Core Mechanics Review (2026-03-11, updated)

## Breaker State Machine: PASS
- Idle -> Dashing -> Braking -> Settling -> Idle matches design doc
- Dash requires directional commitment (velocity or held key)
- Tilt flows naturally through states: dash forward, brake opposite, settle neutral

## Bolt Reflection: PASS
- Direction fully overwritten on breaker contact -- no incoming angle carryover
- hit_fraction + tilt angle + speed preservation all correct
- speed.max(base_speed) prevents momentum death spirals

## Bump System: PASS
- Timing grades correct: Early/Late punish (0.8x), Perfect rewards (1.5x), None neutral (1.0x)
- RESOLVED: grade_bump now uses BoltHitBreaker messages (not bolt_velocity.y proxy)
- IMPLEMENTED: perfect_bump_dash_cancel system exists -- needs schedule ordering verification

## Bolt-Cell Collision: PASS
- AABB + swept raycasting for tunneling + face-based reflection correct
- "One cell per tick" limit fine for Phase 1, revisit for piercing Amps
- MIN_PHYSICS_FPS hardcoded at 30.0 -- acceptable but needs max_speed dependency comment

## Bolt-Lost: CONDITIONAL PASS
- Respawns at min_speed straight up -- functional but no real penalty
- Phase 2 adds breaker-type penalties (lives/time) which will fix this
- Straight-up respawn is actually easiest recovery -- consider random angle

## Min Angle Enforcement: PASS
- ~10deg from horizontal prevents tedious lateral ping-pong
- Preserves speed and sign when correcting

## Speed Management: PASS with note
- Bolt speed floored to base_speed on breaker contact (prevents death spirals)
- No speed decay mechanism -- one perfect bump permanently elevates speed
- Consider per-bounce decay in Phase 2 to reward consistent execution

## Data-Driven Parameters: PASS
- Every gameplay tunable in RON configs
- Rust defaults match RON values
- Tests verify RON parsing for all config types
- Only non-RON constants are UI layout (loading bar) and physics robustness (MIN_PHYSICS_FPS)

## Parameter Assessment
- Perfect window (50ms / ~3 frames at 60fps) is tight but learnable
- Bump anticipation distance ~60 world units at base speed
- Brake tilt > dash tilt creates "which angle do I want" decisions
- Dash covers ~19% playfield width -- emergency repositioning, not safe travel

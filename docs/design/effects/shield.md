# Shield

Spawns a timed visible floor wall at the playfield bottom that blocks bolt losses.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `duration` | `f32` | How long the shield wall persists (seconds) |

## Behavior

Spawns a `ShieldWall` entity — a visible, blue HDR floor wall at the playfield bottom — and attaches a `ShieldWallTimer` set to `duration` seconds.

- The wall uses the same collision layers as permanent walls (membership: `WALL_LAYER`, mask: `BOLT_LAYER`), so bolts bounce off it normally.
- `tick_shield_wall_timer` ticks the timer each `FixedUpdate` (after `BoltSystems::WallCollision`) and despawns the wall when the timer expires.
- If a shield wall already exists when `fire()` is called, the timer is reset to the new `duration` in-place — no second wall is spawned (additive refresh, not stacking).
- The target entity receives no component. The wall is a standalone spawned entity.

## Reversal

Despawns all `ShieldWall` entities immediately.

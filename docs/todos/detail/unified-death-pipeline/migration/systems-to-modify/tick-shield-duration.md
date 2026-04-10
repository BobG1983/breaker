# tick_shield_duration

## Current Behavior
`src/effect/effects/shield/system.rs`

Ticks shield wall timer each frame. When `ShieldDuration` reaches 0 or below, calls `commands.entity(entity).despawn()` directly.

## Migration

Replace direct despawn with `KillYourself<Wall>`:

| Before | After |
|--------|-------|
| `commands.entity(entity).despawn()` | `writer.write(KillYourself::<Wall> { victim: entity, killer: None })` |

Killer is `None` — timer expiry is an environmental death with no dealer.

The system continues to:
- Tick `ShieldDuration` each frame
- Subtract reflection cost when bolts bounce off the shield

The system does NOT:
- Insert `Dead`. The wall domain kill handler does that.
- Despawn the entity. `process_despawn_requests` does that.
- Send `Destroyed<Wall>`. The kill handler does that.

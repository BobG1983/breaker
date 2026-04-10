# despawn_second_wind_on_contact

## Current Behavior
`src/effect/effects/second_wind/system.rs`

When a bolt bounces off a second-wind wall, calls `commands.entity(msg.wall).despawn()` directly.

## Migration

Replace direct despawn with `KillYourself<Wall>`:

| Before | After |
|--------|-------|
| `commands.entity(msg.wall).despawn()` | `writer.write(KillYourself::<Wall> { victim: msg.wall, killer: None })` |

Killer is `None` — the wall self-destructs after serving its purpose.

The system continues to:
- Read bolt-wall collision messages
- Detect when the bounced wall is a SecondWindWall

The system does NOT:
- Insert `Dead`. The wall domain kill handler does that.
- Despawn the entity. `process_despawn_requests` does that.

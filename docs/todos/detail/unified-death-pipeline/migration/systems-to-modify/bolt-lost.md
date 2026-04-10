# bolt_lost

## Current Behavior
`src/bolt/systems/bolt_lost/system.rs`

Detects bolts that leave the play area (below breaker or off-screen). Sends `BoltLost` (for the effect system) and `RequestBoltDestroyed { bolt: entity }` (for cleanup).

## Migration

Replace `RequestBoltDestroyed` with `KillYourself<Bolt>`:

| Before | After |
|--------|-------|
| `writer.write(RequestBoltDestroyed { bolt: entity })` | `writer.write(KillYourself::<Bolt> { victim: entity, killer: None })` |

Killer is `None` — bolt loss is an environmental death with no dealer.

`BoltLost` continues to be sent unchanged — the effect system's `on_bolt_lost_occurred` bridge reads it.

The system continues to:
- Detect bolts below the breaker or outside the playfield
- Send `BoltLost` for the effect system
- Respawn a new bolt (the respawn logic is unchanged)

The system does NOT:
- Insert `Dead`. The domain kill handler does that.
- Despawn the bolt. `process_despawn_requests` does that.

## Message Migration

`RequestBoltDestroyed` is removed entirely once all producers migrate. This system and `tick_bolt_lifespan` are the only producers.

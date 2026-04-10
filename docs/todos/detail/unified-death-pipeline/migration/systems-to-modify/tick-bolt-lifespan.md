# tick_bolt_lifespan

## Current Behavior
`src/bolt/systems/tick_bolt_lifespan.rs`

Ticks `BoltLifespan` timers on bolt entities. When the timer finishes, sends `RequestBoltDestroyed { bolt: entity }`.

## Migration

Replace `RequestBoltDestroyed` with `KillYourself<Bolt>`:

| Before | After |
|--------|-------|
| `writer.write(RequestBoltDestroyed { bolt: entity })` | `writer.write(KillYourself::<Bolt> { victim: entity, killer: None })` |

Killer is `None` — lifespan expiry is an environmental death with no dealer.

The system continues to:
- Tick `BoltLifespan` timers each frame
- Skip bolts with `Birthing` component
- Fire only on `just_finished()`

The system does NOT:
- Insert `Dead`. The domain kill handler does that.
- Despawn the bolt. `process_despawn_requests` does that.

## Message Migration

`RequestBoltDestroyed` is removed entirely once all producers migrate. Consumers:
- `cleanup_destroyed_bolts` — replaced by domain kill handler + `process_despawn_requests`

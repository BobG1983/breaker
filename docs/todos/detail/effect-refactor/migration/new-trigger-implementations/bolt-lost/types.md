# Types

## Message migration

`BoltLost` changes from a unit struct to a struct with fields:

| Before | After |
|--------|-------|
| `BoltLost` (unit struct) | `BoltLost { bolt: Entity, breaker: Entity }` |

The bolt domain's `bolt_lost` system must populate both fields when sending the message.

## Notes

- `BoltLostOccurred` is a Global trigger that carries participants (both bolt and breaker are known).
- The bridge reads `bolt` and `breaker` directly from the message — no query needed.

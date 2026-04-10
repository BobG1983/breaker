# Types

No new types needed. `BoltLost` message already exists.

## Notes

- `BoltLostOccurred` is a Global trigger but carries participants (both bolt and breaker are known).
- `BoltLost` is currently a unit struct. The bridge system may need to query for the breaker entity at dispatch time since it is not carried in the message. If `BoltLost` is extended to carry `bolt: Entity` and `breaker: Entity`, the query becomes unnecessary -- but that is a separate migration concern. The bridge must handle whichever form exists at implementation time.

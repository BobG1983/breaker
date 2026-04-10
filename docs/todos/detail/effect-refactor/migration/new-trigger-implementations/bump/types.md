# Types

No new types needed for bump bridges.

## Existing types consumed

- `BumpPerformed { grade: BumpGrade, bolt: Option<Entity>, breaker: Entity }` — message sent by `grade_bump` on any bump contact (Perfect, Early, Late). Source: `breaker/messages.rs`.
- `BumpWhiffed` — unit message sent by `grade_bump` when a forward bump window expires without bolt contact. Source: `breaker/messages.rs`.
- `BumpGrade { Perfect, Early, Late }` — enum carried by `BumpPerformed`. Source: `breaker/messages.rs`.

## Note on NoBump

There is no existing `NoBump` message. The current `no_bump.rs` bridge is a placeholder stub. The NoBump trigger requires a new message — likely `BoltContactedBreaker` or similar — sent when a bolt hits the breaker with no bump input active. This message does not exist yet. The `on_no_bump_occurred` bridge spec documents what the bridge will do once the message exists; the message itself is out of scope for bridge implementation and belongs to the breaker domain.

# Types

## Existing types consumed

- `BumpPerformed { grade: BumpGrade, bolt: Option<Entity>, breaker: Entity }` — message sent by `grade_bump` on any bump contact (Perfect, Early, Late). Source: `breaker/messages.rs`.
- `BumpWhiffed` — unit message sent by `grade_bump` when a forward bump window expires without bolt contact. Source: `breaker/messages.rs`.
- `BumpGrade { Perfect, Early, Late }` — enum carried by `BumpPerformed`. Source: `breaker/messages.rs`.

## Migration: BoltImpactBreaker change

`BoltImpactBreaker` currently has `{ bolt: Entity, breaker: Entity }`. Add a `bump_status: BumpStatus` field to indicate whether bump input was active at the time of contact.

```rust
enum BumpStatus {
    Active,
    Inactive,
}

struct BoltImpactBreaker {
    bolt: Entity,
    breaker: Entity,
    bump_status: BumpStatus,
}
```

When `bump_status` is `BumpStatus::Inactive`, the `on_no_bump_occurred` bridge dispatches `NoBumpOccurred`. The bolt-breaker collision system sets `bump_status` based on whether bump input was active at contact time.

## Note on NoBump bridge

The `on_no_bump_occurred` bridge reads `BoltImpactBreaker` where `bump_status == BumpStatus::Inactive`. It uses `bolt` and `breaker` from the message to build context. No separate NoBump message needed.

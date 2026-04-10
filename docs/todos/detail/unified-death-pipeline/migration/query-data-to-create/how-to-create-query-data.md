# How to Create QueryData

## The Pattern

QueryData is a named struct that bundles query components so systems don't have giant inline tuple types. Derived with `#[derive(QueryData)]`.

```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BoltSpeedData {
    pub bolt: &'static Bolt,
    pub spatial: SpatialData,        // nested QueryData — composable
    pub active_speed_boosts: Option<&'static ActiveSpeedBoosts>,
}
```

## Mutable vs Read-Only

`#[query_data(mutable)]` generates both a mutable and a read-only variant automatically. The mutable variant has `&'static mut` fields; the read-only variant (suffixed `ReadOnly` by Bevy) converts all `&'static mut` to `&'static`.

Use `#[query_data(mutable)]` when ANY field needs mutation. Systems that only read can use the `ReadOnly` variant of the same QueryData.

If no fields need mutation, omit `#[query_data(mutable)]` — all fields are `&'static` references.

## Nesting

QueryData types can nest inside each other:

```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BoltCollisionData {
    pub entity: Entity,
    pub spatial: SpatialData,           // from rantzsoft_spatial2d
    pub collision: BoltCollisionParams, // another QueryData
}
```

This is fine and encouraged — it reuses existing bundles rather than duplicating field lists.

## Grabbing Extra Read-Only Data

It's okay for a QueryData to include more read-only fields than a specific system needs, as long as the extra fields are `&'static` (not `&'static mut`). Read-only access doesn't hurt parallelism — Bevy can run multiple systems that read the same components concurrently.

Only `&'static mut` fields create exclusive access that blocks parallel execution. Keep mutable fields to the minimum the system actually writes.

## Where They Go

QueryData types live in `queries.rs` (or `queries/data.rs` for domains with many) at the domain root. See `src/bolt/queries.rs` and `src/breaker/queries/data.rs` for existing examples.

Shared QueryData types used across domains go in `src/shared/queries.rs`.

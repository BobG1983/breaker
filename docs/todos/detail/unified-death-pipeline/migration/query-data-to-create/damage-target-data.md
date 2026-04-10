# Name
DamageTargetData

# Filepath
`src/shared/queries.rs`

# Contents
```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct DamageTargetData {
    pub hp: &'static mut Hp,
    pub killed_by: &'static mut KilledBy,
}
```

Used by `apply_damage::<T>` with a `With<T>` filter (and `Without<Locked>` for cells). The same QueryData works for all entity types — the `With<T>` filter is applied at the system level, not in the QueryData.

Mutable because apply_damage writes to both Hp (decrement) and KilledBy (set on killing blow).

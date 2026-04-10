# Name
DeathDetectionData

# Filepath
`src/shared/queries.rs`

# Contents
```rust
#[derive(QueryData)]
pub(crate) struct DeathDetectionData {
    pub entity: Entity,
    pub killed_by: &'static KilledBy,
    pub hp: &'static Hp,
}
```

Used by `detect_cell_deaths`, `detect_bolt_deaths`, `detect_wall_deaths` with a `With<T>` filter and `Changed<Hp>` filter. The same QueryData works for all entity types.

Read-only — detection systems only read Hp and KilledBy. No `#[query_data(mutable)]` needed, which means these systems can run in parallel with each other and with other read-only systems.

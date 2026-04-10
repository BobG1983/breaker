# Name
ComboStreak

# Struct
```rust
#[derive(Resource, Default)]
struct ComboStreak {
    count: u32,
}
```

# Location
`src/effect/resources/`

# Description
Tracks the current consecutive perfect bump streak. Incremented by bump bridge systems when PerfectBumped fires. Reset to zero when any other bump outcome fires (EarlyBumped, LateBumped, BumpWhiffOccurred, NoBumpOccurred).

Read by `is_combo_active(world, threshold)` to evaluate the ComboActive condition for During nodes.

The streak persists across the node — it is NOT reset on node start. A player who chains perfect bumps across multiple nodes maintains their streak.

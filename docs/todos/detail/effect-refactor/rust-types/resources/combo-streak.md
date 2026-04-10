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
`src/effect/conditions/`

# Description
Tracks the current consecutive perfect bump streak. Updated by a dedicated `track_combo_streak` system (not a bridge) that reads `BumpPerformed` messages. Incremented on PerfectBumped. Reset to zero on any other bump outcome (EarlyBumped, LateBumped, BumpWhiffOccurred, NoBumpOccurred). This system runs in FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`.

Read by `is_combo_active(world, threshold)` to evaluate the ComboActive condition for During nodes.

The streak persists across the node — it is NOT reset on node start. A player who chains perfect bumps across multiple nodes maintains their streak.

# Name
is_shield_active

# Signature
```rust
fn is_shield_active(world: &World) -> bool;
```

# Source Location
`src/effect_v3/conditions/shield_active.rs`

# Description
Returns true when at least one entity with the `ShieldWall` component exists in the world. Returns false when no shield walls exist.

This condition becomes true when a Shield effect fires (spawns a ShieldWall) and false when all shield walls are despawned (duration expired, reflection cost depleted, or reversed).

This evaluator does NOT:
- Count shield walls. Any non-zero count is true.
- Distinguish between shields from different sources.
- Modify any state. Pure read-only query.

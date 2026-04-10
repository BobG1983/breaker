# Name
is_combo_active

# Signature
```rust
fn is_combo_active(world: &World, threshold: u32) -> bool;
```

# Source Location
`src/effect/conditions/combo_active.rs`

# Description
Returns true when the current consecutive perfect bump streak is at or above the given threshold. Reads a combo tracking resource from the world.

This condition becomes true when the player chains enough perfect bumps to reach the threshold and false when a non-perfect bump breaks the streak (early, late, whiff, or no bump).

The combo counter is updated by bump bridge systems — when PerfectBumped fires, the counter increments. When any other bump outcome fires, the counter resets to zero. This evaluator only reads the counter.

This evaluator does NOT:
- Modify the combo counter. Read-only.
- Track the streak itself. A separate resource tracks the count.
- Distinguish between different threshold values. Each During(ComboActive(n)) entry passes its own n to this function.

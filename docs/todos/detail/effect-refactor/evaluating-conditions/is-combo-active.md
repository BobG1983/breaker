# Name
is_combo_active

# Signature
```rust
fn is_combo_active(world: &World, threshold: u32) -> bool;
```

# Source Location
`src/effect_v3/conditions/combo_active.rs`

# Description
Returns true when the current consecutive perfect bump streak is at or above the given threshold. Reads `HighlightTracker.consecutive_perfect_bumps` from the world (run domain resource — no effect-domain resource is involved).

This condition becomes true when the player chains enough perfect bumps to reach the threshold and false when a non-perfect bump breaks the streak (early, late, whiff, or no bump).

The combo counter is managed by the run domain's `HighlightTracker`. This evaluator only reads the counter.

This evaluator does NOT:
- Modify the combo counter. Read-only.
- Track the streak itself. `HighlightTracker` owns and updates the count.
- Distinguish between different threshold values. Each `During(ComboActive(n))` entry passes its own `n` to this function.

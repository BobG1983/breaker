# Name
BumpStatus

# Syntax
```rust
enum BumpStatus {
    Active,
    Inactive,
}
```

# Description
- Active: Bump input was active when the bolt contacted the breaker. The bump grading system will grade the timing.
- Inactive: No bump input was active. The `on_no_bump_occurred` bridge dispatches `NoBumpOccurred`.

Added to `BoltImpactBreaker` message as a migration. See `migration/rust-type-swaps.md`.

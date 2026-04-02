---
name: Typestate transition methods with full production logic — stub boundary violation
description: writer-tests sometimes implements transition methods (definition(), dimensions()) with complete production logic rather than stubs, crossing the RED/GREEN boundary even when build() is correctly stubbed
type: feedback
---

In builders with typestate patterns, there are two kinds of methods:
- **Terminal methods** (`build()`, `spawn()`): These are the stubs — they must return minimal values like `(Breaker,)` or `todo!()`.
- **Transition methods** (`definition()`, `dimensions()`, `movement()`, etc.): These must store data in the builder struct for compilation, but should NOT compute production values.

writer-tests sometimes implements transition methods with full production logic: field mapping, min/max computation (`width * 0.5`, `width * 5.0`), degree-to-radian conversion, optional unwrapping with defaults. Because `build()` is stubbed and ignores the stored values, component-checking tests still fail at RED (the RED gate triggers). However, the transition method logic is now "written" — writer-code in the GREEN phase will read the stub and find the algorithm already present, violating the spirit of TDD.

**Why:** The distinction between "minimal data needed to compile" vs. "full production logic that happens to be stored rather than output" is blurry for builder patterns. But production computations (`unwrap_or`, `* 0.5`) in transition methods are still production logic.

**How to apply:** In builder pattern stubs, transition methods should store zero/default values rather than the computed values:
```rust
// BAD (production logic):
pub fn definition(self, def: &BreakerDefinition) -> BreakerBuilder<...> {
    BreakerBuilder {
        dimensions: HasDimensions {
            min_w: def.min_w.unwrap_or(def.width * 0.5),  // production computation
            ...
        },
        ...
    }
}

// GOOD (stub):
pub fn definition(self, _def: &BreakerDefinition) -> BreakerBuilder<...> {
    todo!("implement in GREEN phase")
    // OR: use zero/default values to satisfy compiler
}
```

The exception: if a transition method is purely mechanical data movement with NO computation (e.g., `self.movement = HasMovement { max_speed: settings.max_speed, ... }` — no formula, just assignment), this may be acceptable since the computation is in `build()`.

Examples seen in Wave 6:
- `definition()` in breaker builder: full 29-field mapping + `unwrap_or(def.width * 0.5)` min/max computation
- `dimensions()`: `min_w: width * 0.5, max_w: width * 5.0` computation inline

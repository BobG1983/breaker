---
name: Default impl must call private serde-default fns
description: BreakerDefinition calls private default_* fns in its Default impl; WallDefinition inlines the value instead — the BreakerDefinition pattern is correct
type: project
---

In `BreakerDefinition::default()`, every field with a serde default calls the corresponding private `const fn default_*()`. This keeps the `Default` impl and the serde default in sync.

`WallDefinition::default()` deviates: it inlines `half_thickness: 90.0` directly rather than calling `default_half_thickness()`. This creates a maintenance hazard — if the constant changes, `Default` will not follow.

**Why:** Identified during Wall Builder Pattern Wave 1 review.
**How to apply:** Flag any `Default` impl that repeats the magic value instead of calling the corresponding private default fn.

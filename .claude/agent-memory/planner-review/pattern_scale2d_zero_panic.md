---
name: Scale2D zero panic on initial radius
description: Scale2D::new panics when x or y is zero; any visual effect that starts at radius 0 and grows will panic if animate system sets Scale2D = radius * 2 on frame 0
type: feedback
---

Scale2D::new (in rantzsoft_spatial2d/src/components.rs) asserts both x and y are non-zero. Any spec that uses Scale2D to visualize an expanding radius (shockwave, explosion, pulse) must ensure the initial Scale2D value is never zero.

**Why:** The shockwave rework spec (Phase 8) proposed `Scale2D = current_radius * 2`, but if current starts at 0.0, this produces `Scale2D::uniform(0.0)` which panics.

**How to apply:** When reviewing specs for visual effects that grow from zero size, check that the initial Scale2D value is either (a) set to a small non-zero value (epsilon or 1.0), (b) guarded with a skip-if-zero check, or (c) the initial radius is non-zero.

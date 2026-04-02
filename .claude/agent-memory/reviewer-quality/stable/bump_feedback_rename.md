---
name: BumpFeedback rename pattern
description: The BumpVisualParams → BumpFeedback rename is complete in component names but BreakerConfig fields still use bump_visual_ prefix
type: project
---

The component `BumpVisualParams` was renamed to `BumpFeedback` in the Wave 1 rename. The rename is complete across type identifiers.

However, the `BreakerConfig` struct fields (in `breaker-game/src/breaker/resources.rs`) retain the old naming:
- `bump_visual_duration`
- `bump_visual_peak`
- `bump_visual_peak_fraction`
- `bump_visual_rise_ease`
- `bump_visual_fall_ease`

**Why:** Config fields are tied to RON asset keys and changing them requires updating all `.breaker.ron` files. This appears to be a deliberate deferral.
**How to apply:** Flag as a minor vocabulary drift if seen in future reviews.

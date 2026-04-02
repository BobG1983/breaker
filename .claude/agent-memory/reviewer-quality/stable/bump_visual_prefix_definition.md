---
name: bump_visual_ prefix in BreakerDefinition
description: BreakerDefinition fields use bump_visual_ prefix (bump_visual_duration etc.) — deliberate, matches RON keys
type: project
---

`BreakerDefinition` (in `breaker-game/src/breaker/definition.rs`) retains `bump_visual_duration`, `bump_visual_peak`, `bump_visual_peak_fraction`, `bump_visual_rise_ease`, `bump_visual_fall_ease` field names. This matches the `BreakerConfig` struct naming (which is the RON config resource) and is intentional.

The builder's `BumpFeedbackSettings` intermediary correctly bridges these `bump_visual_*` fields from definition/config into the `BumpFeedback` component.

**Why:** RON asset keys are stable contracts; renaming them would break all existing .breaker.ron files.
**How to apply:** Do not flag `bump_visual_` prefix in `BreakerDefinition` or `BreakerConfig` as a vocabulary issue.

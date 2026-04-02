---
name: Stale helper name in bump_visual tests
description: breaker/systems/bump_visual/tests.rs has default_bump_visual_params() returning BumpFeedback — stale pre-rename name
type: project
---

`breaker-game/src/breaker/systems/bump_visual/tests.rs` has a test helper:
```rust
fn default_bump_visual_params() -> BumpFeedback { ... }
```
The return type is `BumpFeedback` (post-rename) but the function name retains the old `_visual_params` suffix from when the type was called `BumpVisualParams`.

Should be renamed to `default_bump_feedback()`.

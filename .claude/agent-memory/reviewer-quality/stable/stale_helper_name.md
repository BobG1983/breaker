---
name: Stale helper name in bump_visual tests — RESOLVED
description: default_bump_visual_params() was renamed to default_bump_feedback() in feature/breaker-builder-pattern — no open gap
type: project
---

**RESOLVED** as of feature/breaker-builder-pattern (2026-04-02).

`breaker-game/src/breaker/systems/bump_visual/tests.rs` now has:
```rust
fn default_bump_feedback() -> BumpFeedback { ... }
```

The stale `default_bump_visual_params` name no longer exists. No action needed. Do NOT re-flag.

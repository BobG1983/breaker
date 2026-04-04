---
name: Explode struct field rename — RESOLVED
description: explode_chaos.scenario.ron was updated to use damage (not damage_mult) — bug is fixed and closed
type: project
---

`EffectKind::Explode` uses field `damage: f32` (not `damage_mult`). The `explode_chaos.scenario.ron`
was updated on the feature/wall-builder-pattern branch to use `damage: 15.0`.

**Verified fixed:** `breaker-scenario-runner/scenarios/stress/explode_chaos.scenario.ron` line 23
now reads `Do(Explode(range: 64.0, damage: 15.0))`. No residual `damage_mult` usage.

**How to apply:** This was a one-time field rename. If a new RON field is added or renamed on
`EffectKind`, search all `.scenario.ron` files before merging. RON parse errors at scenario
load time are fatal — the scenario can't even run.

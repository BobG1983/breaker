---
name: Explode struct field rename breaks scenario RON
description: explode_chaos.scenario.ron uses damage_mult but EffectKind::Explode now requires damage — scenario file not updated when struct changed
type: project
---

`breaker-scenario-runner/scenarios/stress/explode_chaos.scenario.ron` line 23 uses:
```ron
Do(Explode(range: 64.0, damage_mult: 1.5))
```

But `EffectKind::Explode` in `breaker-game/src/effect/core/types/definitions/enums.rs:363-368` is:
```rust
Explode {
    range: f32,
    damage: f32,
}
```

The `damage_mult` field was renamed to `damage` (or removed and replaced with flat damage). The scenario file was not updated. Fix: update `explode_chaos.scenario.ron` to use `damage: <value>` instead of `damage_mult: 1.5`.

**How to apply:** When `EffectKind` variants change their field names or types, search all `.scenario.ron` files for usages and update them. RON parse errors at scenario load time are fatal — the scenario can't even run.

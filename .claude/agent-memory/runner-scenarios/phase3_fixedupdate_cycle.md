---
name: Phase 3 FixedUpdate scheduling cycle (RESOLVED)
description: All 70 scenarios previously failed with FixedUpdate scheduling cycle from b0b2159; resolved by removing .after(EffectSystems::Recalculate) from consumer systems
type: project
---

All 70 scenarios previously failed with `schedule has 0 before/after cycle(s)` panic at `bevy_ecs::schedule::schedule::Schedule::run` on the `FixedUpdate` schedule.

## Root Cause (archived)

Commit `b0b2159` introduced `EffectSystems::Recalculate.after(EffectSystems::Bridge)` in `EffectPlugin::build()`, plus `.after(EffectSystems::Recalculate)` constraints on `move_breaker` (in `BreakerSystems::Move`) and `prepare_bolt_velocity` (in `BoltSystems::PrepareVelocity`).

This created a directed cycle in `FixedUpdate`:

```
EffectSystems::Bridge
  →after→ EffectSystems::Recalculate         [plugin.rs:14]
  →after→ BreakerSystems::Move               [breaker/plugin.rs:67]
  →after→ BoltSystems::PrepareVelocity       [bolt/plugin.rs:69-71]
  →after→ BoltSystems::CellCollision         [bolt/plugin.rs:75]
  →after→ BoltSystems::BreakerCollision      [bolt/plugin.rs:80]
  →after→ BreakerSystems::GradeBump          [breaker/plugin.rs:74]
  →after→ EffectSystems::Bridge              [triggers/bump.rs:33, and 8 other bump/whiff triggers]
```

## Resolution

Removing `.after(EffectSystems::Recalculate)` from the consumer systems (`move_breaker`, `prepare_bolt_velocity`) broke the cycle. All 70 scenarios pass as of 2026-03-28, including all stress suites (32-run and 16-run variants).

## Status

RESOLVED — 70/70 pass, 0 failures. Coverage parity confirmed (all invariants have self-test coverage, all layouts referenced).

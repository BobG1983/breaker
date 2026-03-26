---
name: Effect Target field is load-bearing for dispatch
description: Target field on SpeedBoost/SizeBoost effects determines which handler branch runs; removing it breaks 5 handler files and 2 dispatch functions
type: project
---

`Effect::SpeedBoost { target: Target, multiplier: f32 }` and `Effect::SizeBoost(Target, f32)` carry a `target` field that controls handler dispatch routing:

Triggered path: `fire_typed_event` -> `SpeedBoostFired { target, ... }` -> `handle_speed_boost` matches on `event.target` (Bolt/AllBolts/Breaker)

Passive path: `fire_passive_event` -> `SpeedBoostApplied { target, ... }` -> `handle_bolt_speed_boost` returns early if `event.target != Target::Bolt`, `handle_breaker_speed_boost` returns early if `event.target != Target::Breaker`

Same pattern for SizeBoost: `SizeBoostApplied { target, ... }` -> `handle_bolt_size_boost` / `handle_width_boost`

Also: `Effect::Attraction(AttractionType, f32)` uses `AttractionType` for routing to Cell/Wall/Breaker attraction.

**Why:** Specs proposing to remove Target from Effect enum variants must provide an alternative routing mechanism that reaches handler observers. The routing info must travel from RON definition through evaluate_node -> fire_typed_event -> handler.

**How to apply:** Reject any spec that removes Target from effects without specifying how handler routing works. The current architecture cannot derive target from context alone.

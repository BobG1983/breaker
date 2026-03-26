---
name: ActiveEffects global resource coupling
description: ActiveEffects resource is read by 10+ bridge systems and written by 2 systems; deleting or migrating requires updating every bridge signature and test_app
type: project
---

ActiveEffects is a global resource (`Res<ActiveEffects>`) read by ALL bridge systems in `effect/triggers/`:
- `bridge_bolt_lost`, `bridge_bump`, `bridge_bump_whiff`, `bridge_cell_impact`, `bridge_breaker_impact`, `bridge_wall_impact`, `bridge_no_bump`, `bridge_cell_death`, `bridge_timer_threshold`

Written by: `init_breaker` (ResMut), `dispatch_chip_effects` (Option<ResMut>)

`bridge_timer_threshold` is the only one using `ResMut` (removes consumed threshold chains).

**Why:** Any spec proposing to replace ActiveEffects with entity-local EffectChains must account for all 10+ bridge system signatures, the helper function `evaluate_active_chains`, and ~30+ test functions that init `ActiveEffects` in test_app builders.

**How to apply:** When reviewing specs that modify effect chain storage, enumerate all consumers. The bridge systems are the primary coupling point.

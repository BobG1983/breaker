---
name: Dispatch pattern ownership and All* target gap
description: Dispatch systems live in entity domains (breaker, cells, chips, wall) not effect domain; chip dispatch wraps non-Breaker targets in When(NodeStart) but misses Once wrapper
type: project
---

Effect dispatch is owned by entity domains, not the effect domain. As of 2026-03-30:
- `dispatch_breaker_effects` in `breaker/systems/` — runs OnEnter(Playing) after NodeSystems::Spawn
- `dispatch_cell_effects` in `cells/systems/` — runs OnEnter(Playing) after NodeSystems::Spawn
- `dispatch_chip_effects` in `chips/systems/` — runs Update during ChipSelect
- `dispatch_wall_effects` in `wall/systems/` — runs OnEnter(Playing) chained after spawn_walls

**Why:** `docs/architecture/effects/dispatch.md` states dispatch is NOT part of the effect domain.

**How to apply:** When reviewing new dispatch logic, verify it lives in the entity domain. New effect types that need dispatch should add code to the owning domain's dispatch system, not to the effect domain.

**Known gap:** `docs/architecture/effects/dispatch.md` documents All* target desugaring as `Once(When(NodeStart, On(All*, permanent: true, children)))` but chip dispatch at `chips/systems/dispatch_chip_effects/system.rs:74-81` wraps as `When(NodeStart, On(target, permanent: true, children))` WITHOUT the `Once` wrapper. Tests match the code behavior (see `tests/desugaring.rs`). This means effects re-fire on every NodeStart rather than only the first. Either the doc or the code needs updating to match.

Additionally, `Target::Bolt` is deferred (same path as AllBolts) instead of resolving to the primary bolt at dispatch time as documented. `Target::Cell` and `Target::Wall` are also deferred (correct — entities don't exist during ChipSelect).

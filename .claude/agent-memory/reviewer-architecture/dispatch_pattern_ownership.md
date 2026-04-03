---
name: Dispatch pattern ownership and All* target gap
description: Dispatch systems live in entity domains (breaker, cells, chips, wall) not effect domain; chip dispatch wraps non-Breaker targets in When(NodeStart) but misses Once wrapper
type: project
---

Effect dispatch is owned by entity domains, not the effect domain. As of 2026-04-02 (post-refactors):
- `dispatch_breaker_effects` — **ELIMINATED** in feature/breaker-builder-pattern; replaced by `spawn_or_reuse_breaker` (Breaker::builder() dispatches effects inline)
- `dispatch_cell_effects` in `state/run/node/systems/` — runs OnEnter(Playing) after NodeSystems::Spawn (moved from `cells/systems/` in state lifecycle refactor)
- `dispatch_chip_effects` in `chips/systems/` — runs Update during ChipSelect
- `dispatch_wall_effects` — **ELIMINATED** in wall-builder-pattern; `spawn_walls` (now at `state/run/node/systems/`) dispatches effects inline via Wall::builder()

**Why:** `docs/architecture/effects/dispatch.md` states dispatch is NOT part of the effect domain.

**How to apply:** When reviewing new dispatch logic, verify it lives in the entity domain. New effect types that need dispatch should add code to the owning domain's dispatch system, not to the effect domain.

**Known gap:** `docs/architecture/effects/dispatch.md` documents All* target desugaring as `Once(When(NodeStart, On(All*, permanent: true, children)))` but chip dispatch at `chips/systems/dispatch_chip_effects/system.rs:74-81` wraps as `When(NodeStart, On(target, permanent: true, children))` WITHOUT the `Once` wrapper. Tests match the code behavior (see `tests/desugaring.rs`). This means effects re-fire on every NodeStart rather than only the first. Either the doc or the code needs updating to match.

Additionally, `Target::Bolt` is deferred (same path as AllBolts) instead of resolving to the primary bolt at dispatch time as documented. `Target::Cell` and `Target::Wall` are also deferred (correct — entities don't exist during ChipSelect).

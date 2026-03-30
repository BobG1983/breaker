---
name: Dispatch pattern ownership and All* target gap
description: Dispatch systems live in entity domains (breaker, cells, chips, wall) not effect domain; All* desugaring documented but not implemented in chip dispatch
type: project
---

Effect dispatch is owned by entity domains, not the effect domain. As of 2026-03-29:
- `dispatch_breaker_effects` in `breaker/systems/` — runs OnEnter(Playing) after NodeSystems::Spawn
- `dispatch_cell_effects` in `cells/systems/` — runs OnEnter(Playing) after NodeSystems::Spawn
- `dispatch_chip_effects` in `chips/systems/` — runs Update during ChipSelect
- `dispatch_wall_effects` in `wall/systems/` — runs OnEnter(Playing) chained after spawn_walls

**Why:** `docs/architecture/effects/dispatch.md` states dispatch is NOT part of the effect domain.

**How to apply:** When reviewing new dispatch logic, verify it lives in the entity domain. New effect types that need dispatch should add code to the owning domain's dispatch system, not to the effect domain.

**Known gap:** `docs/architecture/effects/dispatch.md` documents All* target desugaring (`Once(When(NodeStart, On(All*, ...)))`) but no dispatch system implements it. Chip dispatch during ChipSelect resolves Cell/Wall targets to empty sets (entities don't exist yet). Breaker and cell dispatch run after spawn, so they're unaffected.

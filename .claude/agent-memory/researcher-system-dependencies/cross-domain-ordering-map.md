---
name: Cross-domain ordering map
description: All system sets defined in one domain but referenced in another for .after()/.before() ordering — critical for crate-splitting feasibility
type: project
---

# Cross-Domain System Set References (Bevy 0.18.1)

Analyzed 2026-04-01 for game-crate-splitting feasibility.

## Sets that cross domain boundaries

| Set | Defined in | Referenced by |
|-----|-----------|---------------|
| `BoltSystems::CellCollision` | bolt | effect (10+ bridge systems) |
| `BoltSystems::WallCollision` | bolt | effect (second_wind) |
| `BoltSystems::BreakerCollision` | bolt | breaker (grade_bump), effect (bridges) |
| `BoltSystems::BoltLost` | bolt | effect (bridge_bolt_lost) |
| `BoltSystems::Reset` | bolt | breaker (reset_bolt anchor) |
| `BreakerSystems::Move` | breaker | bolt (hover_bolt, bolt_cell_collision) |
| `BreakerSystems::GradeBump` | breaker | effect (bridge_bump), run (detect_close_save) |
| `BreakerSystems::Reset` | breaker | bolt (reset_bolt ordering) |
| `BreakerSystems::InitParams` | breaker | bolt, breaker (OnEnter) |
| `NodeSystems::Spawn` | run::node | bolt, breaker (OnEnter ordering) |
| `NodeSystems::TrackCompletion` | run::node | run (3 systems) |
| `NodeSystems::ApplyTimePenalty` | run::node | run (handle_timer_expired) |
| `EffectSystems::Bridge` | effect | bolt (before + after), cells (cleanup_cell after) |
| `PhysicsSystems::MaintainQuadtree` | rantzsoft_physics2d | bolt, effect (5+ systems) |
| `PhysicsSystems::EnforceDistanceConstraints` | rantzsoft_physics2d | bolt (2 systems) |

## Critical circular dependencies (for splitting)

- bolt ↔ breaker ↔ effect: each references the other's system sets
- effect → cells: cells::cleanup_cell.after(EffectSystems::Bridge)
- bolt → dispatch_bolt_effects.before(EffectSystems::Bridge)

## Why: feeds into feasibility analysis at `.claude/todos/detail/game-crate-splitting/research/system-ordering-constraints.md`
## How to apply: when evaluating any crate-splitting plan, check whether the set types involved here are in a shared crate or would create circular deps

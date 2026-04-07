---
name: Phase 7 findings — post-import-refactor scan
description: Wave 11 scan (2026-04-06 develop, post-import-refactor): 0 HIGH, 11 MEDIUM actionable — ALL resolved in feature/scenario-runner-wiring. Monitor and pure-prod files remain.
type: project
---

Updated 2026-04-07: All 11 MEDIUM actionable items confirmed resolved in feature/scenario-runner-wiring.

## Scope

Full workspace scan: all .rs files across breaker-game, rantzsoft_lifecycle, rantzsoft_spatial2d, rantzsoft_physics2d, rantzsoft_defaults, breaker-scenario-runner (1176 files total, Wave 11 2026-04-06).

## Status vs Phase 6

All Phase 6 HIGH items resolved:
- orchestration.rs (was 1302) split into module directory
- fade.rs (was 1032) split into module directory
- effects/mod.rs violation cleaned (37 lines now)
- All lifecycle effect files (wipe, iris, pixelate, dissolve, slide) split

## MEDIUM actionable — ALL RESOLVED (feature/scenario-runner-wiring)

All 11 files confirmed split into module directories as of 2026-04-07:
launch_bolt → launch_bolt/, sync_breaker_scale → sync_breaker_scale/, bump → bump/,
enforce_distance_constraints → enforce_distance_constraints/, maintain_quadtree → maintain_quadtree/,
cells/resources.rs → refactored away entirely, bolt/registry → registry/,
breaker_wall_collision → breaker_wall_collision/, sync_bolt_scale → sync_bolt_scale/,
breaker/definition → definition/, lifecycle/route → route/

## Monitor approaching 800-line Strategy C threshold

- reset_bolt/tests.rs (762), snapshot_node_highlights/tests.rs (749), impact/context_entity_tests.rs (730), impacted/context_entity_tests.rs (722)

## Pure production files to monitor (unsplittable)

- execution.rs (493), tether_beam/effect.rs (488), quadtree/tree.rs (487), terminal.rs (475), definitions.rs (472), enums/types.rs (433)

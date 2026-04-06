---
name: Phase 7 findings — post-import-refactor scan
description: Wave 11 scan (2026-04-06 develop, post-import-refactor): 0 HIGH, 11 MEDIUM actionable (all Strategy A inline tests), 40+ monitor, 0 mod.rs violations. Phase 6 HIGH items all resolved. Spec at .claude/specs/file-splits.md
type: project
---

Updated 2026-04-06 on develop branch (Wave 11, post-import-refactor scan).

## Scope

Full workspace scan: all .rs files across breaker-game, rantzsoft_lifecycle, rantzsoft_spatial2d, rantzsoft_physics2d, rantzsoft_defaults, breaker-scenario-runner (1176 files total).

## Status vs Phase 6

All Phase 6 HIGH items resolved:
- orchestration.rs (was 1302) split into module directory
- fade.rs (was 1032) split into module directory
- effects/mod.rs violation cleaned (37 lines now)
- All lifecycle effect files (wipe, iris, pixelate, dissolve, slide) split

New prelude/ files all well under threshold (5-25 lines each).

## MEDIUM actionable (11 files, all Strategy A)

launch_bolt.rs (449), sync_breaker_scale.rs (441), bump.rs (435), enforce_distance_constraints.rs (436), maintain_quadtree.rs (453), cells/resources.rs (430), bolt/registry.rs (415), breaker_wall_collision.rs (434), sync_bolt_scale.rs (407), breaker/definition.rs (406), lifecycle/route.rs (412)

## Monitor approaching 800-line Strategy C threshold

- reset_bolt/tests.rs (762), snapshot_node_highlights/tests.rs (749), impact/context_entity_tests.rs (730), impacted/context_entity_tests.rs (722)

## Pure production files to monitor

- execution.rs (493), tether_beam/effect.rs (488), quadtree/tree.rs (487), terminal.rs (475), definitions.rs (472), enums/types.rs (433)

---
name: impact_bolt_impact_breaker
description: Full reference map for BoltImpactBreaker — adding a `breaker: Entity` field (2026-03-28)
type: project
---

Complete reference map captured 2026-03-28. Context: adding `breaker: Entity` field.

## Definition
- `breaker-game/src/bolt/messages.rs:28-31` — `pub(crate) struct BoltImpactBreaker { pub bolt: Entity }`
  Derives: `Message, Clone, Debug`. Visibility: `pub(crate)`.

## Registration
- `breaker-game/src/bolt/plugin.rs:39` — `app.add_message::<BoltImpactBreaker>()`

## Producers (send sites)
- `breaker-game/src/bolt/systems/bolt_breaker_collision/system.rs:120` — overlap resolution path
- `breaker-game/src/bolt/systems/bolt_breaker_collision/system.rs:169` — CCD top/bottom hit path
  Both use `writer.write(BoltImpactBreaker { bolt: bolt_entity })` — no `breaker` field yet.

## Consumers (read sites)
- `breaker-game/src/breaker/systems/bump/system.rs:113` — `grade_bump` reads via `MessageReader<BoltImpactBreaker>`
  Uses `hit.bolt` at line 132 (forward path) and line 139 (retroactive path stores `Some(hit.bolt)`).

## Test constructors (all sites that build the struct)
- `breaker-game/src/bolt/messages.rs:161` — debug format test in `#[cfg(test)]` module
- `breaker-game/src/bolt/systems/bolt_breaker_collision/tests/helpers.rs:101-106` — `TestHitMessage` resource storing it; `collect_breaker_hit_bolts` reads `msg.bolt`
- `breaker-game/src/breaker/systems/bump/tests/helpers.rs:101` — `TestHitMessage(pub Option<BoltImpactBreaker>)` resource type
- `breaker-game/src/breaker/systems/bump/tests/helpers.rs:104-109` — `enqueue_hit` writes the message from `TestHitMessage`
- `breaker-game/src/breaker/systems/bump/tests/helpers.rs:112-129` — `grade_bump_test_app()` calls `app.add_message::<BoltImpactBreaker>()`
- `breaker-game/src/breaker/systems/bump/tests/helpers.rs:139` — `combined_bump_test_app()` calls `app.add_message::<BoltImpactBreaker>()`
- `breaker-game/src/breaker/systems/bump/tests/grade_bump.rs:31-33` — constructs `BoltImpactBreaker { bolt: Entity::PLACEHOLDER }`
- `breaker-game/src/breaker/systems/bump/tests/grade_bump.rs:69-71` — constructs with PLACEHOLDER
- `breaker-game/src/breaker/systems/bump/tests/grade_bump.rs:97-99` — constructs with PLACEHOLDER
- `breaker-game/src/breaker/systems/bump/tests/grade_bump.rs:156-158` — constructs with specific `bolt_entity`
- `breaker-game/src/breaker/systems/bump/tests/grade_bump.rs:205-207` — constructs with specific `bolt_entity`
- `breaker-game/src/breaker/systems/bump/tests/grade_bump.rs:255-257` — constructs with PLACEHOLDER (ForceBumpGrade test)
- `breaker-game/src/breaker/systems/bump/tests/grade_bump.rs:289-291` — constructs with PLACEHOLDER
- `breaker-game/src/breaker/systems/bump/tests/grade_bump.rs:323-325` — constructs with PLACEHOLDER
- `breaker-game/src/breaker/systems/bump/tests/combined.rs:32-34` — constructs with PLACEHOLDER
- `breaker-game/src/breaker/systems/bump/tests/forward_bump.rs:235` — `app.add_message::<crate::bolt::messages::BoltImpactBreaker>()` (no struct literal)

## Stale doc reference
- `docs/architecture/messages.md:13` — uses old name `BoltHitBreaker { bolt }` — doc is stale

## Scenario runner
- No references to BoltImpactBreaker in the scenario runner codebase.

## Effect domain
- All trigger bridge systems (impact, impacted, no_bump, etc.) are stubs — none reference BoltImpactBreaker.

## Adding `breaker: Entity` field — change surface
Every `BoltImpactBreaker { bolt: ... }` struct literal must become `BoltImpactBreaker { bolt: ..., breaker: <breaker_entity> }`.
The two send sites in `bolt_breaker_collision/system.rs` already have `breaker_query.single()` returning the breaker entity — but the result is destructured and the entity is not bound. A variable will need to be introduced.
All test constructors using `Entity::PLACEHOLDER` need a second field added. The tests that use a specific bolt entity (grade_bump.rs lines 156, 205) do not use the breaker entity at all — they can use `Entity::PLACEHOLDER` for the new field.

**Why:** Adding `breaker: Entity` field to BoltImpactBreaker (2026-03-28).
**How to apply:** This map is ephemeral — recheck before acting, as production and test code will change.

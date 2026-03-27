# Collision Detection and Impact Messages

Collision detection lives in the **entity domains**, not the effect domain. Each domain detects its own collisions and sends Bevy messages that the Impact/Impacted trigger systems listen for.

## Collision Messages

| Collision | Detecting domain | Message |
|-----------|-----------------|---------|
| Bolt ↔ Cell | `bolt/` | `BoltImpactCell { bolt, cell }` |
| Bolt ↔ Wall | `bolt/` | `BoltImpactWall { bolt, wall }` |
| Bolt ↔ Breaker | `bolt/` | `BoltImpactBreaker { bolt, breaker }` |
| Breaker ↔ Cell | `breaker/` | `BreakerImpactCell { breaker, cell }` |
| Breaker ↔ Wall | `breaker/` | `BreakerImpactWall { breaker, wall }` |
| Cell ↔ Wall | `cells/` | `CellImpactWall { cell, wall }` |

Messages are defined in the **detecting domain** (the domain that runs the collision system). The `Impact` and `Impacted` trigger systems in `effect/triggers/` listen for these messages.

## Impact → Trigger Mapping

A single collision message produces four triggers. Example — `BoltImpactCell { bolt, cell }`:

1. `Impact(Cell)` — global sweep ("an impact with a cell happened")
2. `Impact(Bolt)` — global sweep ("an impact with a bolt happened")
3. `Impacted(Cell)` — targeted on the bolt ("you were in an impact with a cell")
4. `Impacted(Bolt)` — targeted on the cell ("you were in an impact with a bolt")

## Implementation Status

**Existing collision systems** (need message rename + split):
- `bolt/systems/bolt_cell_collision` — currently handles both bolt↔cell AND bolt↔wall. Split into two separate systems:
  - `bolt/systems/bolt_cell_collision` — sends `BoltImpactCell` (was `BoltHitCell`)
  - `bolt/systems/bolt_wall_collision` — sends `BoltImpactWall` (was `BoltHitWall`)
- `bolt/systems/bolt_breaker_collision` — sends `BoltImpactBreaker` (was `BoltHitBreaker`)

**New collision systems** (add as part of this refactor):
- `breaker/` — `BreakerImpactCell` (breaker ↔ cell collision detection)
- `breaker/` — `BreakerImpactWall` (breaker ↔ wall collision detection)
- `cells/` — `CellImpactWall` (cell ↔ wall collision detection, for future moving cells)

The new systems should NOT be minimal stubs initially (even though there are no moving cells or breaker-cell collisions yet), make sure the messages and trigger bridge systems should exist so the effect system is wired up and ready.

## Adding a New Collision Type

See [Adding Collisions](adding_collisions.md).

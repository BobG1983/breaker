---
name: Survival Wave 6A performance patterns
description: suppress_bolt_immune_damage and kill_bump_vulnerable_cells — allocs, query filters, archetype fragmentation
type: project
---

## suppress_bolt_immune_damage

File: `breaker-game/src/cells/behaviors/survival/systems/suppress_bolt_immune_damage.rs`

Structural mirror of `check_armor_direction` (Wave 3). All patterns were already accepted there.

### Blocklist Vec alloc
`Vec::new()` per FixedUpdate tick unconditionally. `Vec::new()` does not heap-allocate until
the first push, so this is free when no BoltImmune cells exist. Fast-path guard at line 27
returns early if blocklist is empty, so the drain+collect only runs on ticks with immune hits.
Pattern is identical to `check_armor_direction` line 43/72. Acceptable.

### drain().collect() is borrow-checker-forced
Same reasoning as armored: `drain()` and `write()` both take `&mut self` on `Messages<T>`;
collecting first is the only correct pattern. Only runs when blocklist is non-empty. Acceptable.

### Query: With<BoltImmune> — correct and narrow
`Query<(), With<BoltImmune>>` narrows to exactly the survival archetype. Single-entity
`immune_query.get(impact.cell)` inside the impact loop (1 impact/tick at current 1-bolt scale).
No loop-wide iter(), no unnecessary mutable access. Correct.

### Message volume
`BoltImpactCell` fires once per bolt-cell collision. At 1 bolt + ~50–200 cells, the loop runs
0–1 times per tick. Impact negligible.

---

## kill_bump_vulnerable_cells

File: `breaker-game/src/cells/behaviors/survival/systems/kill_bump_vulnerable_cells.rs`

### Query: (With<BumpVulnerable>, Without<Dead>) — correct
Filter pair is correct: narrows to living survival cells only. `Without<Dead>` prevents double-
kill messages from being written to already-dead entities. Single-entity `vulnerable_query.get()`
inside the breaker-impact loop (rare event: breaker hits a cell).

### Mutable access: MessageWriter only, no &mut Component
All query access is read-only. No unnecessary mutable component borrows. Parallelism not blocked.

### Message volume
`BreakerImpactCell` fires only on breaker-cell contact — a rare, low-frequency event. Zero cost
in the typical case.

---

## Archetype fragmentation: BoltImmune + BumpVulnerable + SurvivalTurret + SurvivalPattern + SurvivalTimer

All five components are inserted together at spawn time in `terminal.rs` via a single `.insert((...))`
call for `CellBehavior::Survival`. `SurvivalPermanent` omits `SurvivalTimer` — this produces a
second archetype variant, but both are stamped at spawn and never mutated at runtime (no churn).
No archetype invalidation per frame. Acceptable at any grid size.

---

## Overall: all patterns clean. No action needed.

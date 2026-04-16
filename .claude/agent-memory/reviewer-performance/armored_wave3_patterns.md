---
name: Armored Wave 3 performance patterns
description: check_armor_direction system — blocklist Vec, drain+collect, ArmoredCell archetype, query filters
type: project
---

## check_armor_direction (Wave 3 Armored)

File: `breaker-game/src/cells/behaviors/armored/systems/check_armor_direction.rs`

### Blocklist Vec alloc (per tick, unconditional)

`Vec::new()` on line 43 every FixedUpdate tick, even when no armored cells exist. At current scale
(few armored cells, 0–5 impacts/tick) this is negligible — `Vec::new()` doesn't heap-allocate until
the first push. Acceptable; could use `Local<Vec<...>>` in future if allocator pressure matters.

### drain().collect() is borrow-checker-forced

`damage.drain().collect()` on line 83 materializes all pending DamageDealt<Cell> into a heap Vec
before iterating. This is **required** — `drain()` takes `&mut self` on `Messages<T>` and
`write()` also takes `&mut self`, so you cannot drain and write back in a single iterator pass.
The collect-first pattern is correct, not a fixable anti-pattern. Only runs when blocklist is
non-empty (fast-path guard at line 72 returns early). Mark as acceptable/intentional.

### Archetype: clean, no churn

ArmoredCell + ArmorValue + ArmorFacing are inserted together at spawn via
`terminal.rs:173-175` (CellBehavior::Armored match arm) and **never removed at runtime**.
No archetype fragmentation. ArmorValue ranges 1–3 and ArmorFacing has 4 variants — these
create at most 12 distinct armored archetypes but with ~50–200 total cells, fragmentation
is academic.

### Query filter: correct

ArmorQuery uses `With<ArmoredCell>` to narrow to the armored archetype only. `armor_query.get(impact.cell)`
is a single entity lookup inside the impact loop (0–5 per tick). This is correct — no loop-wide
query.iter() needed; the entity handle from the message drives the lookup.

### bolt_query: &mut correctly required

`BoltPiercingQuery` uses `&mut PiercingRemaining` — required because breakthrough decrements `pr.0`.
Immutable access would not suffice.

### Overall: all patterns acceptable at current scale. No action needed.

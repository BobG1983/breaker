---
name: impact_multi_bolt
description: Full reference map for MultiBolt/multi_bolt/multi-bolt — located in one docs file only, absent from all code (2026-03-28)
type: project
---

## Search Summary

Searched across: all `.rs` source files in breaker-game, breaker-scenario-runner, and all rantzsoft_* crates; all `.ron` asset and scenario files; all `docs/**/*.md` design and architecture docs; all `.claude/agent-memory/**/*.md` memory files.

`rg` (`Grep` tool) was unavailable; search was performed by reading all known file paths exhaustively.

## Confirmed References

### docs/design/chip-catalog.md:391
- **Context**: Supernova evolution effect description in the chip catalog doc
- **Relationship**: documents
- **Exact text**: `Do(MultiBolt(base_count: 2, count_per_level: 0, stacks: 1)), Do(Shockwave(base_range: 96, speed: 400))`
- **Note**: This is stale — the actual `supernova.evolution.ron` already uses `SpawnBolts(count: 2, inherit: true)`, not `MultiBolt`. The doc has not been updated.

### docs/design/evolutions.md:41
- **Context**: Evolution categories table, "Offensive" category examples column
- **Relationship**: documents (generic concept, not an identifier)
- **Exact text**: `multi-bolt burst`
- **Note**: This is lowercase hyphenated — it's a plain English phrase, not a type name or identifier. It describes the concept abstractly, not the removed `MultiBolt` type.

## Code Search Results

No references found in:
- `breaker-game/src/**/*.rs` — `EffectKind` enum has no `MultiBolt` variant (uses `SpawnBolts`)
- `breaker-game/assets/chips/evolution/supernova.evolution.ron` — uses `SpawnBolts(count: 2, inherit: true)`
- `breaker-scenario-runner/src/**/*.rs` — no reference
- `breaker-scenario-runner/scenarios/**/*.ron` — no reference
- All `rantzsoft_*` crates — no reference
- All agent memory files — no reference

## Action Required

To remove all references:

1. **docs/design/chip-catalog.md:391** — Update the Supernova evolution effect to match the actual RON:
   - Replace `Do(MultiBolt(base_count: 2, count_per_level: 0, stacks: 1)), Do(Shockwave(...))`
   - With `Do(SpawnBolts(count: 2, inherit: true)), Do(Shockwave(...))`

2. **docs/design/evolutions.md:41** — Optional: the phrase "multi-bolt burst" is generic English and not an identifier reference. It does not need to change unless removing all conceptual mentions is desired.

## Safety Assessment

Safe to update — no code references, no RON data references, no test references. Only docs need updating.

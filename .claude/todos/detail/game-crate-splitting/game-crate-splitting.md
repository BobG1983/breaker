# Game Crate Splitting

## Summary
Investigate splitting `breaker-game` monolith into sub-crates to improve compile times and enforce domain boundaries at the type level.

## Context
`breaker-game` is a single crate containing all game domains (bolt, breaker, cells, chips, effect, run, fx, audio, ui, debug, input, wall, screen, shared). 783 files, 104k lines across 14 domains. The `effect` domain is the largest at 34k lines.

## Research Findings

Research completed 2026-04-01. Full reports in [research/](research/).

### Compile Times ([baselines](research/compile-time-baselines.md))
- Clean check: 1m 24s (dominated by bevy dependency compilation)
- Incremental check after touching any file: ~0.9s
- No-op incremental: 0.6s
- **Verdict**: Incremental type-checks are already sub-second. Compile-time gains from splitting would be negligible for `cargo dcheck`. Codegen (`cargo dbuild`/`cargo dtest`) might tell a different story — not yet measured.

### Cross-Domain Dependencies ([full map](research/cross-domain-dependencies.md))
Three hard circular dependency cycles:
1. **bolt ↔ breaker** — `spawn_bolt` reads breaker registries; `grade_bump` reads bolt impact data
2. **bolt ↔ effect** — effects spawn/mutate bolts; bolt reads active effect states. Tightest cycle.
3. **cells ↔ effect** — cell queries include effect components; effects trigger cell destruction

Two structural blockers:
- **Entity markers** (`Bolt`, `Breaker`, `Cell`, `Wall`) used as `With<X>` filters across 6-8 domains each — must move to a shared crate
- **RootEffect embedding** — `BoltDefinition`, `BreakerDefinition`, `CellTypeDefinition`, `ChipDefinition` all embed `Vec<RootEffect>` — definition crates structurally depend on effect core types

### System Ordering ([full map](research/system-ordering-constraints.md))
- 23 explicit cross-domain ordering constraints
- bolt ↔ breaker ↔ effect triangle references each other's `SystemSet` enums
- Systems querying multiple domains: `bolt_cell_collision`, `bolt_breaker_collision`, `dispatch_chip_effects`, `tick_tether_beam`
- Stale ordering anchor: `apply_gravity_pull` and `apply_attraction` have no anchor after `BoltSystems::PrepareVelocity` was eliminated

### Clean Split Candidates
These domains have no cycles to resolve: `input`, `audio`, `fx`, `wall`, `screen`, `debug`, `shared`

### Hard Core (tightly coupled)
These form a cycle mesh: `bolt`, `breaker`, `cells`, `chips`, `effect`, `run`

## Scope
- In: Investigation (done), proposed split boundaries, migration plan if worthwhile
- Out: Actually performing the split

## Dependencies
- Depends on: Phase 5 completion (visual identity will add a `rantzsoft_vfx` crate and restructure `fx`/`ui`/`screen`)
- Blocks: Phases 6+ (audio, content, roguelite) would benefit from cleaner boundaries

## Open Questions
1. Do `cargo dbuild`/`cargo dtest` codegen times tell a different story than `cargo dcheck`?
2. Is the organizational benefit (enforced domain boundaries) worth the migration cost even without compile-time gains?
3. Should the hard core (bolt/breaker/cells/chips/effect/run) stay as one crate with only leaf domains extracted?
4. What's the right granularity — `breaker-core` + leaf crates, or more fine-grained?

## Status
`[NEEDS DETAIL]` — Research complete. Missing: codegen time measurements, cost/benefit decision, proposed architecture if proceeding

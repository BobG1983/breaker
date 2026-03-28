---
name: terminology
description: Confirmed term mappings and glossary gaps for the brickbreaker roguelite (2026-03-28)
type: project
---

## Confirmed Correct Terminology in Code

- `BoltImpactCell`, `BoltImpactWall`, `BoltImpactBreaker` — correct collision message names (not BoltHit*)
- `BreakerImpactCell`, `BreakerImpactWall` — breaker collision messages added in collision-cleanup branch
- `CellImpactWall` — cell collision message
- `DamageCell.source_chip` — attribution field (not `source_bolt`)
- `EffectKind::SecondWind` — unit variant (no fields)
- `EffectKind::EntropyEngine { max_effects, pool }` — field is `max_effects`, NOT `threshold`
- `BoundEffects` — permanent chains on entities (not `ActiveEffects` or `ArmedEffects`)
- `StagedEffects` — one-shot chains on entities (not `EffectChains`)
- `EffectCommandsExt` — Commands extension for firing/reversing effects
- `SpawnBolts` — correct effect name (not `MultiBolt`)

## Intentional Shorthand in Docs (not drift)

- chip-catalog.md trigger notation: `When(PerfectBumped)`, `When(OnBump)`, `When(OnBoltLost)` — authoring shorthand matching Trigger enum variants loosely. `OnBump` = `Bump`, `OnBoltLost` = `BoltLost`, `OnPerfectBump` = `PerfectBump`. These are design docs, not RON syntax specs.

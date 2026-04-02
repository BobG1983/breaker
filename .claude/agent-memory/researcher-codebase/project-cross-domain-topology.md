---
name: breaker-game cross-domain topology
description: Summary of cross-domain coupling in breaker-game, key cycles, and split feasibility tiers
type: project
---

The `breaker-game` monolith has 14 domains. The `shared` module is already a foundation layer (no cycles). The key coupling facts worth recalling:

- **bolt ↔ breaker**: tight cycle — spawn_bolt reads BreakerRegistry/SelectedBreaker; grade_bump reads BoltImpactBreaker. Cannot split without a shared interface or moving entity markers to breaker-shared.
- **bolt ↔ effect**: tight cycle — effect effects spawn/modify bolts (spawn_bolts, gravity_well); bolt uses effect component types (BoundEffects, ActiveSpeedBoosts, etc.).
- **cells ↔ effect**: bidirectional via ShieldActive (effect→cells component) and DamageCell/CellDestroyedAt (cells messages read by effect triggers).
- **Entity marker problem**: Bolt, Breaker, Cell, Wall are used as With<X> query filters across nearly every domain. Moving them to breaker-shared is the prerequisite for any meaningful split.
- **effect::core types** (RootEffect, BoundEffects, StagedEffects, EffectNode) appear in BoltDefinition, BreakerDefinition, CellTypeDefinition, ChipDefinition — so all four definition-holding domains depend on effect for their own data types.
- **screen** is a pure consumer (reads from 8+ domains, nothing reads from screen). Natural leaf crate candidate.
- **fx** is minimally coupled — depends only on shared. Receives from bolt/breaker/run but nothing structural.
- **input** has zero incoming cross-domain deps. Trivially splittable.

The full analysis is at `.claude/todos/detail/game-crate-splitting/research/cross-domain-dependencies.md`.

**Why:** This was researched 2026-04-01 to evaluate feasibility of splitting breaker-game into sub-crates for compile time improvements.
**How to apply:** Use this to scope any sub-crate splitting work — Tier 1 (shared, input, fx, audio) can happen independently; Tier 3 (bolt, breaker, effect) requires interface design first.

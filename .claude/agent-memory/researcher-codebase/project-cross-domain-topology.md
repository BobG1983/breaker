---
name: breaker-game cross-domain topology
description: Summary of cross-domain coupling in breaker-game, key cycles, and split feasibility tiers
type: project
---

The `breaker-game` monolith has 14 domains. The `shared` module is already a foundation layer (no cycles). The key coupling facts worth recalling:

- **bolt ↔ breaker**: tight cycle — spawn_bolt reads BreakerRegistry/SelectedBreaker; grade_bump reads BoltImpactBreaker. Cannot split without a shared interface or moving entity markers to breaker-shared.
- **bolt ↔ effect**: tight cycle — effect effects spawn/modify bolts (spawn_bolts, gravity_well); bolt uses effect component types (BoundEffects, ActiveSpeedBoosts, etc.).
- **cells ↔ effect**: bidirectional via DamageCell/CellDestroyedAt (cells messages read by effect triggers). Note: `ShieldActive` cross-domain coupling was ELIMINATED in the Shield refactor (2026-04-02) — Shield is now a timed visible floor wall entity; no cross-domain component writes.
- **Entity marker problem**: Bolt, Breaker, Cell, Wall are used as With<X> query filters across nearly every domain. Moving them to breaker-shared is the prerequisite for any meaningful split.
- **effect::core types** (RootEffect, BoundEffects, StagedEffects, EffectNode) appear in BoltDefinition, BreakerDefinition, CellTypeDefinition, ChipDefinition — so all four definition-holding domains depend on effect for their own data types.
- **screen domain ELIMINATED** (state lifecycle refactor Wave 1, 2026-04-02): screen/ → state/. UI subdomains moved to state/run/node/hud/, state/run/chip_select/, state/menu/. `screen` as "natural leaf crate candidate" is no longer applicable — the domain no longer exists as a unit.
- **fx** is minimally coupled — depends only on shared. Receives from bolt/breaker/state/run but nothing structural.
- **input** has zero incoming cross-domain deps. Trivially splittable.

The full analysis is at `docs/todos/detail/game-crate-splitting/research/cross-domain-dependencies.md`.

**Why:** This was researched 2026-04-01 to evaluate feasibility of splitting breaker-game into sub-crates for compile time improvements.
**How to apply:** Use this to scope any sub-crate splitting work — Tier 1 (shared, input, fx, audio) can happen independently; Tier 3 (bolt, breaker, effect) requires interface design first.

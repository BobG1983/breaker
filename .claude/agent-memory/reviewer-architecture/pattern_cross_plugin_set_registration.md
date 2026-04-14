---
name: Cross-plugin system-set registration (phase set)
description: Plugin A registers systems into a SystemSet owned by Plugin B when the set represents a pipeline phase, not a specific system anchor
type: project
---

# Cross-plugin system-set registration — the "phase set" precedent

## The pattern

`RunPlugin` registers `handle_breaker_death.in_set(DeathPipelineSystems::HandleKill)` into a
SystemSet defined and `configure_sets`-configured by `DeathPipelinePlugin`
(`shared/death_pipeline/plugin.rs`).

This is the FIRST production instance of one plugin tagging a system into another plugin's
SystemSet. Previously, cross-plugin interactions used `.after(OtherSystems::Variant)` only.

## Why it is acceptable here

`DeathPipelineSystems::{ApplyDamage, DetectDeaths, HandleKill}` is a **phase set**, not a
"named pivotal system" set per `docs/architecture/ordering.md`. Each variant represents a
stage of the damage -> death -> despawn pipeline, and multiple plugins may legitimately
contribute systems to the same stage (e.g., generic `handle_kill::<T>` for Cell/Bolt/Wall
plus specialized `handle_breaker_death` for Breaker).

Alternatives are worse:

1. **Moving `handle_breaker_death` into `shared/death_pipeline/systems/`** would force
   shared/death_pipeline to import `state::run::messages::RunLost` — boundary inversion
   where foundational shared code depends on a consumer domain.
2. **Using `.after(DeathPipelineSystems::DetectDeaths)` without `.in_set`** loses the
   semantic information that the system belongs in the HandleKill phase, and creates
   asymmetry with the generic handlers.

## Build-order note

Bevy 0.18 resolves system ordering at schedule initialization, not during `Plugin::build`.
It is safe for `RunPlugin::build` (called inside `StatePlugin`, added BEFORE
`DeathPipelinePlugin` in `game.rs`) to register a system `.in_set(DeathPipelineSystems::HandleKill)`
even though `DeathPipelinePlugin::build` runs later and is the caller of `configure_sets`.

## Tension with ordering.md

`docs/architecture/ordering.md` currently says:
> "Each variant names one pivotal system that other domains depend on."
> "Consuming domains order with `.after(DomainSystems::Variant)`."

The new pattern violates the letter of these rules but not the spirit. If this pattern
stays, `ordering.md` should be updated to carve out a "phase set" exception for pipeline
stage sets (currently only `DeathPipelineSystems`).

## Visibility

`DeathPipelineSystems` is `pub(crate)` — intra-crate visibility is sufficient. No export
beyond the game crate is needed.

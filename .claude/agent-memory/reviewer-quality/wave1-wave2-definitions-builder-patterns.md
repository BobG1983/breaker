---
name: Wave 1+2 Definition/Builder Review — Established Patterns
description: Patterns from review of bolt/definition.rs, registry.rs, builder/core.rs, components/definitions.rs, resources.rs, breaker/definition.rs, screen/plugin.rs
type: project
---

## `bolt_definition_clone_preserves_fields` does not call `.clone()`

`definition.rs:187` assigns `let cloned = def;` — this is a move, not a clone call. The test
name claims it is testing `Clone` but it does not exercise the trait at all (the struct derives
`Clone` so `.clone()` would produce a separate heap allocation for the String and Vec fields).
Flag as [Fix] — the test title is misleading and the Clone path is untested.

By contrast, `bolt_definition_clone_with_effects_preserves_entries` (line 211) has the same
pattern: `let cloned = def;` with no `.clone()` call. Both should use `def.clone()`.

## `build()` terminal methods in builder/core.rs still lack `#[must_use]`

Confirmed as of this review: none of the four `build()` impls
(`BoltBuilder<...,Serving,Primary>::build`, `...,Serving,Extra>::build`,
`...,HasVelocity,Primary>::build`, `...,HasVelocity,Extra>::build`) have `#[must_use]`.
`spawn()` methods consume `self` so silently ignoring their return would be an unusual mistake,
but `build()` returns a bundle — silently dropping it is a real footgun.
This is an established [Fix]-level finding from the builder-migration-patterns.md memory.

## `has_explicit` / `has_inherited` bool pattern in `spawn_inner` — confirmed Nit

`builder/core.rs:411-413` computes two booleans then immediately checks the same Options inside
the branch. This is redundant — the booleans add no information over checking `optional.with_effects.is_some()`.
Confirmed [Nit] from builder-migration-patterns.md.

## `BoltServing` doc comment says "player" — acceptable vocabulary in prose

`definitions.rs:16`: "waiting for the player to launch it" — "player" in plain-English doc prose
is acceptable (there is no game-vocabulary term for the human at the keyboard).
Do NOT flag this as a vocabulary violation.

## `DEFAULT_RADIUS` constant in builder/core.rs is private and undocumented — confirmed Nit

`builder/core.rs:33`: `const DEFAULT_RADIUS: f32 = 8.0;` has a doc comment.
No issue — this was not a finding.

## `BoltConfig::initial_angle` unit inconsistency — radians in struct, degrees in BoltDefinition

`resources.rs:37`: `initial_angle` doc says "in radians". `BoltDefinition.min_angle_horizontal/vertical`
docs say "in degrees". `builder/core.rs:234-236` converts the BoltDefinition degree values with
`.to_radians()` but treats `config.initial_angle` as already in radians (no conversion).
This is correct behavior — but the two config structs use different units for angle fields
without a clear naming convention (no `_deg` or `_rad` suffix). The inconsistency is a [Debt]
item for future naming alignment.

## `spawn_with_empty_inherited_effects_inserts_empty_bound_effects` — edge case gap resolved

`spawn_and_effects_tests.rs:195`: tests that an empty `BoundEffects` results in a `BoundEffects`
component being present. This is a subtle behavior: `with_inherited_effects(&BoundEffects(vec![]))`
will enter the `if has_explicit || has_inherited` branch and insert `BoundEffects(vec![])`.
The test covers this. No gap here.

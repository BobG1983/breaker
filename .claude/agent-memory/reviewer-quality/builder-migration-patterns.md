---
name: Bolt Builder Migration — Established Patterns
description: Patterns from the chip-evolution-ecosystem builder migration review: must_use gaps, post-spawn lifespan insertion, arch doc discrepancy, and intentional designs
type: project
---

## `BoltBuilder` terminal methods lack `#[must_use]`

All four `build()` impls and four `spawn()` impls in `breaker-game/src/bolt/builder.rs` are missing
`#[must_use]`. Only `Bolt::builder()` itself (line 95) has `#[must_use]`. The `SpatialDataBuilder`
in `rantzsoft_spatial2d/src/builder.rs` correctly annotates every `build()` method. This is a
real [Fix]-level issue: silently discarding a built bundle produces an invisible bug.

**How to apply:** Always flag missing `#[must_use]` on builder `build()` and `spawn()` terminals.
Do NOT flag the entry point annotation — it is present.

## `spawn_bolts/effect.rs` and `spawn_phantom/effect.rs` post-spawn lifespan insertion is Debt

Both effects insert `BoltLifespan` manually after calling `.spawn(world)`:
- `spawn_bolts/effect.rs:54-58` — `world.entity_mut(bolt_entity).insert(BoltLifespan(...))`
- `spawn_phantom/effect.rs:104` — `world.entity_mut(phantom).insert(BoltLifespan(...))`

The builder already provides `.with_lifespan(f32)` which does the same thing. These post-spawn
insertions duplicate builder logic and will silently diverge if `BoltLifespan` construction changes.
Flag as [Debt] — the `spawn_phantom` case also inserts `PiercingRemaining` and `PhantomBoltMarker`
post-spawn so cannot be fully collapsed into the builder call, but `BoltLifespan` specifically can move.

## `Bolt::builder()` is not `const fn` — inconsistent with `Spatial::builder()`

`Spatial::builder()` is `pub const fn`. `Bolt::builder()` is `pub fn` only. The bolt builder
initialises `OptionalBoltData::default()` which uses `Option::None` fields — const-compatible in
theory. This is a [Nit]: making it `const fn` would require a manual `const fn default()` impl on
`OptionalBoltData` rather than `#[derive(Default)]`.

## Architecture doc says `pub fn new()` — actual builders use `builder()`

`docs/architecture/type_state_builder_pattern.md:72` Conventions section states the entry point
convention as `pub fn new()`. Both `Bolt::builder()` and `Spatial::builder()` use `builder()`.
This is stale doc — flag as [Fix] since it will mislead future builder implementations.

## `has_explicit` / `has_inherited` boolean pattern in `spawn_inner`

Lines 356-367 in builder.rs compute `has_explicit` and `has_inherited` booleans and then match
the same Options again inside the branch. This is a [Nit] — the booleans are redundant. Prefer
building the vec unconditionally and checking `!effect_entries.is_empty()`.

## Test coverage gaps confirmed

- `BoltBuilder<HasPosition, HasSpeed, HasAngle, Serving, Extra>::build()` — no test verifying
  `BoltServing` component is present. Serving+Extra build() path is untested.
- `.with_inherited_effects()` alone (without `.with_effects()`) — no test.
- `spawn_bolts inherit=false` with existing BoundEffects — previously flagged [Fix], still missing
  (wave3-tether-chain-spawn-bolts-patterns.md).

## `spawned_by(&str)` calling `.to_string()` is intentional

`SpawnedByEvolution(pub String)` requires a heap allocation. The `.to_string()` call in
`spawned_by(&str)` is required, not a smell. Do NOT flag.

## All typestate markers in bolt builder are `pub` — correct

The markers (`NoPosition`, `HasPosition`, etc.) appear in return types of public methods, so they
must be `pub`. Their fields are private (e.g. `pos: Vec2`). This matches the arch doc convention.
`BoltBuilder<P,S,A,M,R>` is `pub` with the module as `pub(crate)` — correct per the convention.

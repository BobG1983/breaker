---
name: Source Chip / Shield Dispatch — Intentional Patterns
description: Patterns from the feature/source-chip-shield-absorption branch dispatch systems that look like violations but are intentional
type: project
---

## `Target::Bolt` vs `Target::AllBolts` asymmetry in dispatch_cell_effects

In `dispatch_cell_effects`, `Target::Bolt` maps to `.iter().next()` (first bolt only)
while `Target::AllBolts` maps to `.iter().collect()` (all bolts). This is intentional:
cell-defined effects targeting `Target::Bolt` mean "the bolt that hit me" — a single-bolt
semantic that cannot be resolved at dispatch time without context, so first-bolt is the
conservative fallback. `dispatch_breaker_effects` and `dispatch_chip_effects` both treat
`Target::Bolt` and `Target::AllBolts` identically (collect all bolts), because breaker/chip
effects are applied uniformly across all bolts. Do not flag the asymmetry without checking
both dispatch call sites.

## `PushBoundEffects` is `pub(crate)` intentionally

`PushBoundEffects` in `commands.rs` is `pub(crate)` rather than private. It is used
indirectly via `commands.push_bound_effects()` in dispatch systems. `pub(crate)` exposes
it for potential direct command queue use in tests or sibling modules. Accept as intentional.

## `dispatch_wall_effects` is a `const fn` stub

`dispatch_wall_effects` is declared `pub(crate) const fn`. This is a known stub: wall
definitions do not currently carry effects. The `const fn` declaration is valid for a no-op
system that takes no mutable world access. Do not flag as missing implementation.

## `BypassExtras` SystemParam in lifecycle/systems.rs

`BypassExtras` is a `SystemParam` struct that bundles four fields extracted from
`bypass_menu_to_playing` to keep it under the 7-argument clippy system param limit.
The pub fields are needed by the system. Accept as intentional ergonomic extraction.

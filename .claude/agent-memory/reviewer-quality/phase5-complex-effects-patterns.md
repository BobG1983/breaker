---
name: Phase 5 Complex Effects — Intentional Patterns
description: Patterns established in the Phase 5 feature/runtime-effects complex effects that look like violations but are correct for this codebase
type: project
---

## `TetherBeamComponent` naming with "Component" suffix

`TetherBeamComponent` carries the "Component" suffix, which is unusual (most components in this codebase don't use that suffix). However it disambiguates from the effect-type name `TetherBeam` in the `EffectKind` enum and the module name `tether_beam`. Do NOT flag without first checking whether `TetherBeam` is also a type in `EffectKind`.

**Why:** The module is named `tether_beam`, the effect kind is `TetherBeam`, so `TetherBeamComponent` prevents a name collision for the Bevy component that holds the runtime state.

**How to apply:** Accept `TetherBeamComponent` as intentional disambiguation. If `TetherBeam` is renamed in `EffectKind`, then `TetherBeamComponent` could be revisited.

## `kill_count` in `EntropyEngineState`

`kill_count` uses the word "kill" which is not a project vocabulary term. The correct vocabulary for cells destroyed is `cells_destroyed` (as in `RunStats::cells_destroyed`). However `kill_count` is scoped entirely within the `EntropyEngineState` component and refers specifically to the escalation counter for the EntropyEngine chip (cells destroyed this node). The terminology docs don't explicitly prohibit "kill" for component-local counters. Flag as a vocabulary issue but note it's minor — the correct field name would be `cells_destroyed` per codebase convention.

## Bolt spawn deduplication in `spawn_bolts.rs` and `tether_beam.rs`

Both `spawn_bolts.rs` and `tether_beam.rs` duplicate the full bolt-component tuple spawn pattern. This is consistent with the plugin-per-domain convention (effect handlers are self-contained). Do not flag as redundancy without confirming a shared bolt-spawning helper exists or is planned.

## `unwrap()` in tether_beam.rs tests at lines 348-351

The four `world.get::<Bolt*>(*bolt).unwrap()` calls are in `#[cfg(test)]` test code. Per project convention, `unwrap()` is acceptable in tests.

## `beam_half_width` computed from `Option<Res<BoltConfig>>` with `BoltConfig::default()`

`tick_tether_beam` reads `bolt_config: Option<Res<BoltConfig>>` and falls back to `BoltConfig::default().radius`. This is because `BoltConfig` may not be present in all test worlds. The `Option<Res<BoltConfig>>` + `map_or(default)` pattern is intentional defensive coding for system-world compat. Do not flag as unnecessary Option wrapping.

## Comment markers `// Scope for immutable borrow of quadtree` in chain_lightning.rs

The borrow-scope comment at line 48 of chain_lightning.rs explains why candidates are collected into a Vec before the RNG borrow: Rust cannot hold both `&CollisionQuadtree` and `&mut GameRng` simultaneously from `world`. This is an intentional safety comment, not a stale comment.

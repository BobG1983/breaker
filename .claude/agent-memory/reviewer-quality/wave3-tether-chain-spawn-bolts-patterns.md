---
name: Wave 3 TetherBeam Chain Mode and SpawnBolts Inherit — Patterns
description: Patterns from Wave 3 (feature/scenario-coverage) tether_beam chain mode and spawn_bolts inherit fix that look like violations but are intentional
type: project
---

## `chain: bool` parameter in tether_beam::fire() / reverse()

`fire()` and `reverse()` take `chain: bool` as a positional parameter. This directly mirrors the
`EffectKind::TetherBeam { damage_mult: f32, chain: bool }` variant field. The parameter is clear
in context. Flagged as [Debt] (two-variant enum would be more self-documenting at call sites), but
the current form is accepted given the small function count and clear naming.

## Per-beam `HashSet::new()` in `tick_tether_beam`

`damaged_this_tick: HashSet<Entity>` is allocated inside the per-beam loop in `tick_tether_beam`
(effect.rs line 225). With typical 1–3 active beams, this is not hot. Flagged as [Debt] — hoisting
with `clear()` would be cleaner but is not required now.

## Duplicate `tick()` helper in `maintain_chain_tests.rs`

`maintain_chain_tests.rs` defines its own `tick()` function (lines 12-18) that is identical to the
one in `tests/mod.rs` (lines 62-68). The local definition shadows `super::*`. This matches the
co-located test helper pattern established in Phase 1 (scenario-coverage-patterns.md) — do NOT flag.

## `EffectSourceChip(chain_active.source_chip.clone())` in maintain_tether_chain

The `.clone()` on `chain_active.source_chip` (Option<String>) at effect.rs line 283 is required:
`chain_active` is a `ResMut` borrow and cannot be moved out of. The clone is bounded by a
single `Option<String>` per rebuild. Do NOT flag as unnecessary clone.

## Missing Behavior 6 in `fire_inherit.rs`

The test file `spawn_bolts/tests/fire_inherit.rs` numbers behaviors 1–5 and 7, skipping 6. The
missing behavior is likely "inherit=false does NOT copy BoundEffects even when they exist on the
primary bolt." This is a [Fix]-severity test gap — flag it. Do not accept as intentional.

## Probabilistic velocity direction test in fire_tests.rs

`fire_spawns_two_bolts_with_different_velocity_directions` (fire_tests.rs line 105) asserts that
two tether bolts have different velocity directions. The test uses `GameRng::default()` (deterministic
seed) and relies on the two RNG draws producing different angles. This is a [Nit] — the assertion
works in practice because the seed is known-safe, but coupling to RNG output is fragile. Flag it.

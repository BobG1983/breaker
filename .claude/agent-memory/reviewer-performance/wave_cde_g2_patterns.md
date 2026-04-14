---
name: Wave C/D/E/G2 effect_v3 performance patterns
description: Patterns reviewed in Wave C/D/E/G2 — staged effects, armed fire, watcher systems, entropy engine, evaluate_conditions
type: project
---

## RemoveStagedEffectCommand — O(n) scan + remove

`remove_staged.rs`: `iter().position()` + `remove(pos)` on `StagedEffects.0` (Vec).
- O(n) scan + O(n) shift. Acceptable: StagedEffects is tiny (1–3 entries per entity).
- Only fires during command flush after trigger events — not per-frame.
- At Phase 3 if staged entries accumulate (deep chains), watch this. Current: negligible.

## ArmedFiredParticipants::track — HashMap mutation per fire

`armed_fired_participants.rs`: `entry().or_default().push()` inside `TrackArmedFireCommand::apply`.
- Single HashMap insert + Vec push per armed On fire. Acceptable at current entity counts.
- Only fires in command flush after armed On triggers match. Not per-frame baseline cost.

## walk_staged_effects — no early return when slice is empty

`walk_effects.rs`: `walk_staged_effects` iterates `trees` slice directly — if `staged` is None,
callers pass `staged.map(|s| s.0.clone()).unwrap_or_default()` which is an empty Vec.
Walking an empty Vec is O(1) (the for loop body never executes). No issue.

The `.unwrap_or_default()` does allocate an empty Vec (on the heap, but zero-capacity optimization
applies for Vec::new() in Rust — no actual alloc). Not a concern.

## Option<&StagedEffects> in query — NOT archetype fragmentation concern

All 6 bridge files query `(&BoundEffects, Option<&StagedEffects>)`. This widens the archetype
match to cover entities both with and without StagedEffects. At current scale this is fine.
Phase 3 concern: if entities-without-StagedEffects are the large population and entities-with
are the minority, a `With<StagedEffects>` filter path would be more efficient — but this would
require the bridge to handle two separate queries, complicating the design. The Optional pattern
is deliberately chosen here and acceptable.

## on_impact_occurred global dispatch — N_kinds × M_entities inner loop

`impact/bridges.rs`: For each collision event, 3 EntityKind entries are pushed to `kinds`
(specific, specific, Any). Then for each kind, `bound_query.iter()` walks all BoundEffects
entities. With 1 BoltImpactCell event: 3 full entity scans. With 3 events: 9 full scans.
At 50 entities that's 450 entity visits per frame with 3 collisions.
Phase 3 concern at 200+ BoundEffects entities. Current: acceptable.

## Watcher systems — nested loop over registry entries per new entity

`stamp_spawned_*.rs`: For each new entity, iterate all SpawnStampRegistry entries and
filter by EntityKind. Loop is O(registry_entries * new_entities). Registry checked first with
`is_empty()` early return. New-entity events are spawn-time only, not per frame.
Pattern is clean.

## tick_entropy_engine — Option<&EffectSourceChip> query

`entropy_engine/systems.rs`: `Query<(Entity, &mut EntropyCounter, Option<&EffectSourceChip>)>`.
Optional component on the entity that also has EntropyCounter. Very few entities have
EntropyCounter (1 per equipped entropy chip). Fragmentation is academic at this scale.
Confirmed same pattern as other EffectSourceChip optionals — acceptable.

## reverse_scoped_tree ScopedTree::On — commands.reverse_effect in loop

`evaluate_conditions.rs` line 140-142: drains tracked participants, then iterates and calls
`commands.reverse_effect(participant, ...)` per participant. N reverses for N fires.
This is intentional and correct. Only runs on condition-off transitions, not per-frame.
At current scale (1 breaker, few participants per armed On), negligible.

## evaluate_conditions — Vec alloc per frame

`evaluate_conditions.rs` line 26: `Vec::new()` allocated every frame for `during_entries`.
This is in a World-exclusive system that runs every FixedUpdate. Only Breaker entity has
BoundEffects with During nodes currently. Minor allocation per frame. Watch at Phase 3
if cells get During effects.

## staged.0.clone() per entity per bridge iteration

`staged.map(|s| s.0.clone()).unwrap_or_default()` in every global bridge for every entity
when staged is Some. StagedEffects typically has 1–3 entries. At current scale (1 breaker,
rarely any staged entries), negligible. Phase 3 concern same as bound.0.clone().

---
name: Pulse ring performance patterns
description: tick_pulse/tick_pulse_ring/apply_pulse_damage/despawn — structural mirror of shockwave, all patterns acceptable
type: project
---

Wave 4 pulse systems are a faithful structural copy of Wave 3 shockwave.

**Archetype:** PulseRing entities receive 10 components in one spawn() call — no mid-life inserts
except CleanupOnExit removal at node teardown. Single archetype; no fragmentation.

**Allocations at spawn (tick_pulse fire path):**
- `HashSet::new()` — once per ring spawn, gated by `interval`. Unavoidable for hit-dedup semantics.
- `emitter.source_chip.clone()` — `Option<String>` clone once per ring spawn. Cheaper than the
  per-cell-hit clone in tether_beam. Bounded by interval; acceptable.

**tick_pulse query mutability:** `PulseEmitter` query is `&mut` because timer decrements every tick.
A `Changed<>` guard would not narrow this — the component changes every tick by definition.
At 1 emitter, trivially cheap.

**apply_pulse_damage:** O(rings × cells) nested loop identical to shockwave. `HashSet::contains`
dedup means already-hit cells exit immediately without a clone or write. At 1-few active rings and
~50-200 cells, acceptable at current and Phase 3 scale.

**Option<&EffectSourceChip> in PulseRingQuery:** All rings receive EffectSourceChip unconditionally
at spawn, so in practice one archetype is matched. The Option<> is defensive/mirrors shockwave.
Only becomes a concern if spawn bundle is revised to make EffectSourceChip conditional.

**EffectStack::aggregate():** Called once per ring spawn (not per frame). Cheap at any stack depth
players can realistically reach.

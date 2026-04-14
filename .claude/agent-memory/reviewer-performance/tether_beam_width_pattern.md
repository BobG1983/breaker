---
name: TetherBeamWidth component and tick_tether_beam query pattern
description: TetherBeamWidth added uniformly to beam archetype; 5-tuple required query on few short-lived entities; all clean
type: project
---

**TetherBeamWidth archetype impact (Wave 5/6):** `TetherBeamWidth(pub f32)` added to every
beam entity at spawn (both `fire_spawn` and `fire_chain` paths). Since the component is stamped
unconditionally alongside `TetherBeamSource` and `TetherBeamDamage`, all beam entities share one
archetype. No fragmentation introduced — uniform addition to a uniform spawn tuple.

**tick_tether_beam query:** Now a 5-tuple:
`(Entity, &TetherBeamSource, &TetherBeamDamage, &TetherBeamWidth, Option<&EffectSourceChip>)`
The `Option<&EffectSourceChip>` makes this a two-archetype match (with/without chip). Beam count
is 1–few short-lived entities at any moment. The outer loop is O(beams), inner loop is O(cells,
~50–200). No hot-path concern at this scale. Per-beam cost is 1 f32 field read (`beam_width_comp.0`).

**No per-tick allocations added:** The width read is a direct field deref. `chip.and_then(|c| c.0.clone())`
was already present (clones an `Option<String>` per hit cell); no new clone paths introduced.

**Confirmed clean** — no fragmentation concern, no allocation concern at current beam/cell count.

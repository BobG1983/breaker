---
name: EffectSourceChip clone pattern in damage systems
description: chip.and_then(|c| c.0.clone()) per damaged cell — confirmed required, hoist is correct
type: project
---

`EffectSourceChip(Option<String>)` added to shockwave and tether_beam entities. The query binds
`Option<&EffectSourceChip>` once per effect entity (outer loop), not per cell. Inside the inner
cell loop, `chip.and_then(|c| c.0.clone())` clones the `Option<String>` only on the
damage-emit branch (inside the `distance <= radius` or `along/across` guard).

The clone is required: `DamageDealt<Cell>::source_chip: Option<String>` is owned; a lifetime on
`DamageDealt` would break `MessageWriter` compatibility.

**Hoisting is correct:** The `chip` binding is read from the outer per-effect-entity loop.
It is NOT re-fetched per cell. Pattern matches the established chain_lightning reference.

**Shockwave cost:** At most 1 shockwave active at a time. Clone fires once per cell that enters
the radius for the first time (ShockwaveDamaged dedup prevents repeat hits). At 200 cells,
worst case 200 clones over the life of the shockwave, not 200 per tick.

**Tether beam cost:** Fires every FixedUpdate tick for each cell inside the beam corridor.
With 1 tether beam and ~5–10 cells in the corridor, that is 5–10 String clones per tick.
At current scale (few bolts -> few beams), negligible.

**Phase 3 watch:** If tether beam count grows to 3–5 active beams with 20+ cells each,
that is 60–100 clones per FixedUpdate tick. Still small in absolute terms but worth noting.

**`fire_spawn` / `fire_chain`:** `source.to_owned()` inside `from_source` allocates once at
spawn time. Acceptable.

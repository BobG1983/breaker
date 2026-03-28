---
name: EntropyEngine Redesign
description: EntropyEngine redesign from counter-gated random to escalating multi-effect per trigger — cascading chaos
type: project
---

## Decision: Redesign EntropyEngine to Escalating Multi-Effect

**Why:** Old EntropyEngine (counter-gated RandomEffect) was functionally identical to RandomEffect with a delay. Not evolution-worthy — no new interaction points, no escalation, no skill expression.

**New design:** Every DestroyedCell trigger fires `effects_per_fire + (escalation_counter * escalation_per_fire)` random effects from pool, then increments escalation_counter. Counter resets per node.

**How to apply:**
- Parameters: pool (weighted effects), effects_per_fire (base: 2), escalation_per_fire (base: 1)
- First cell kill = 2 random effects. Fifth = 6 effects. Tenth = 11 effects.
- Pool: Shockwave (0.25), SpawnBolts (0.20), ChainLightning (0.20), ChainBolt (0.20), SpeedBoost (0.15)
- Ingredients unchanged: Cascade x1 + Flux x1 (achievable by first boss)
- Node-reset creates "earn the chaos again" tension per node
- Key synergies: Piercing (more kills = faster escalation), Cascade (chain reactions), Splinter

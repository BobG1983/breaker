# 5q: Evolution VFX — Chain/Spawn

## Summary

Implement bespoke VFX for four evolutions that involve chain reactions, entity spawning, recursive effects, and randomness: Shock Chain, Split Decision, Circuit Breaker, and Entropy Engine. These evolutions share a theme of producing cascading or multiplying effects, so their VFX must clearly communicate "something is building/multiplying" while remaining distinct from each other.

## Context

Key architecture changes from the LEGACY plan:
- **No recipe system.** Each evolution VFX is a direct Rust function/system.
- **rantzsoft_particles2d** for particle effects (energy rings, sparks, glow motes, trails).
- **rantzsoft_postprocess** for screen effects (chromatic aberration, screen flash, screen shake).
- **Visual types from visuals/ domain** (5e) — Hue, GlowParams, VisualModifier, ModifierStack.
- **DR-9 corrections**:
  - Entropy Engine has no counter gauge (mechanic has no counter). VFX is prismatic flash per cell destroy, then selected random effect fires. Bolt has persistent prismatic shimmer.
  - "Chain Reaction" in the old mapping refers to the evolution Shock Chain (Chain Reaction is an ingredient chip name).
- **Unimplemented mechanics**: Shock Chain's recursive shockwave mechanic exists in RON but the recursive kill-triggers-more-shockwaves behavior may not be fully wired. Circuit Breaker's `CircuitBreaker` effect leaf exists in RON. Both have RON files — whether the underlying mechanics are complete is a Phase 7 concern. VFX should work with whatever the mechanic produces.

Current RON files:
- `chain_reaction.evolution.ron` (name: "Shock Chain"): CellDestroyed -> `Do(Shockwave(base_range: 64.0, stacks: 1, speed: 400.0))` — Ingredients: Chain Reaction x1 + Aftershock x2 + Cascade x2
- `split_decision.evolution.ron` (name: "Split Decision"): CellDestroyed -> `Do(SpawnBolts(count: 2, inherit: true))` — Ingredients: Splinter x2 + Piercing Shot x2
- `circuit_breaker.evolution.ron` (name: "Circuit Breaker"): PerfectBumped -> `Do(CircuitBreaker(bumps_required: 3, spawn_count: 1, inherit: true, shockwave_range: 160.0, shockwave_speed: 500.0))` — Ingredients: Overclock x1 + Bump Force x2
- `entropy_engine.evolution.ron` (name: "Entropy Engine"): CellDestroyed -> `Do(EntropyEngine(max_effects: 3, pool: [...]))` — Ingredients: Cascade x2 + Flux x2

## Evolutions in This Batch

### Shock Chain

**Mechanic summary**: Cell destruction triggers a shockwave (range 64, speed 400). If that shockwave kills cells, those kills trigger more shockwaves. Recursive chain continues until no more kills occur or depth cap is reached.

**VFX design (from DR-9 and effects-particles.md)**:
- Recursive shockwaves with escalating visual intensity per generation depth.
- 1st generation: base-level ring brightness and spark count.
- 2nd generation: brighter ring, more sparks, slight chromatic aberration.
- 3rd+ generation: intense ring (HDR >1.8), heavy sparks, strong chromatic aberration.
- Chromatic aberration scales with generation depth (deeper = more aberration).
- Screen shake scales with total chain size (3+ cells = small, 6+ = medium, 10+ = heavy).
- The escalation communicates "this is getting out of control" — the deeper the chain, the more visually overwhelming it becomes.

**What to implement**:
1. Generation-depth tracking: the Shock Chain VFX function needs to know which generation a shockwave belongs to. This requires either tagging shockwave entities with generation depth or tracking it through the effect attribution system.
2. Per-generation VFX scaling: ring HDR = base + (generation * 0.3), spark count = base + (generation * 4), ring color shifts warmer with depth.
3. Per-generation chromatic aberration: `TriggerChromaticAberration` with intensity = generation * 0.1, duration 0.2s.
4. Aggregate screen shake: system that tracks total Shock Chain kills in a short window and triggers shake at appropriate tier.

### Split Decision

**Mechanic summary**: Cell destruction spawns 2 permanent bolts that inherit the parent bolt's effects. The newly spawned bolts carry forward the full `BoundEffects` chain.

**VFX design (from DR-9 and effects-particles.md)**:
- Cell fission effect: the target cell's glow visually splits along an axis before the cell is destroyed.
- Two halves condense into bolt-shaped orbs over ~0.15s.
- Energy filaments (thin bright lines) connect the two halves during the split animation.
- Brief flash at the split moment.
- Spawned bolts inherit the parent bolt's visual modifiers (from visuals/ domain ModifierStack).
- Prismatic birth trail particles on spawned bolts (fades after ~0.3s).
- The fission visual communicates transformation — a cell becoming two bolts.

**What to implement**:
1. Cell fission animation system: when Split Decision triggers, play a split animation on the cell entity before it despawns — cell glow divides along a random axis, halves drift apart briefly.
2. Bolt condensation: the two halves animate into bolt-shaped orbs (scale down, shift to bolt color).
3. Energy filament entities: thin beam-like lines connecting the two halves during the ~0.15s split. Filaments fade as halves separate.
4. Flash + sparks at split moment: screen flash (subtle) + RadialBurst at cell position.
5. Prismatic birth trail: spawned bolts receive a timed `VisualModifier` (prismatic color cycle trail, auto-removes after ~0.3s).
6. Visual modifier inheritance: ensure spawned bolts copy the parent bolt's `ModifierStack` so they look consistent.

### Circuit Breaker

**Mechanic summary**: Tracks perfect bumps. Every 3 perfect bumps "close the circuit" — spawns 1 inheriting bolt + large shockwave (range 160, speed 500), then resets. The `CircuitBreaker` effect leaf handles counter tracking internally.

**VFX design (from DR-9 and effects-particles.md)**:
- Persistent three-node triangle charge indicator rendered near the bolt.
- Three small connected dots forming a triangle. Each starts dim.
- Each perfect bump lights one node (dim -> bright), with a small flash on the lit node.
- On completion (3rd bump): all three nodes flash white-hot (HDR >1.5), collapse inward toward center, circuit visually "closes."
- Payoff: spawn bolt + amplified shockwave (larger/brighter than base) + screen flash (Gold color) + medium screen shake.
- The charge phase is subtle and ambient; the payoff is dramatic and satisfying.
- Charge indicator resets to dim after payoff.

**What to implement**:
1. Triangle charge indicator entities: three small glow entities arranged in an equilateral triangle, anchored to follow the bolt. Rendered as faint connected dots with dim connecting lines.
2. Charge system: listens for perfect bump events attributed to Circuit Breaker. On each bump, brightens the next dim node (HDR ramp from ~0.2 to ~1.0) with a small flash particle.
3. Circuit close animation: on 3rd bump, all three nodes flash to HDR >1.5, animate collapse inward (position lerp toward center over ~0.1s), then despawn with a burst of Gold sparks.
4. Payoff VFX: enhanced shockwave (range 160, HDR 2.0, Gold color) + screen flash (Gold, intensity 0.8, ~3 frames) + medium screen shake.
5. Reset: after payoff, re-spawn the triangle indicator at dim state for the next cycle.
6. Cleanup: if the bolt entity is despawned, the triangle indicator entities are also despawned.

### Entropy Engine

**Mechanic summary**: Every cell destroyed fires 1 of 4 random effects from a weighted pool (SpawnBolts 30%, Shockwave 25%, ChainBolt 25%, SpeedBoost 20%). Escalates up to max_effects: 3. No counter, no accumulation — fires on every cell kill.

**VFX design (from DR-9 and effects-particles.md)**:
- Brief prismatic flash (~0.1s) on each cell destroy — 3-4 spectral color cycle (rapid red->gold->blue->violet) before resolving to the selected effect's visual.
- Multi-colored spark starburst from entity on selection.
- Bolt has a persistent prismatic shimmer while Entropy Engine is active — distinguishes it from normal bolts.
- The per-trigger flash is fast and subtle. Entropy Engine modifies other effects — the randomness IS the visual identity, not a single spectacle.
- No counter gauge (mechanic has no counter to display).

**What to implement**:
1. Prismatic flash function: on each Entropy Engine trigger, spawn a brief multi-colored particle burst at cell position (~12 sparks, prismatic color cycle over ~0.1s). Fires BEFORE the selected random effect fires its own VFX.
2. Selected effect VFX: the randomly selected effect (Shockwave, SpawnBolts, etc.) fires its own base VFX normally. Entropy Engine only adds the prismatic flash pre-effect.
3. Persistent bolt shimmer: when Entropy Engine is active on a bolt, apply a `VisualModifier` for color cycling (slow prismatic cycle, ~3 Hz, rotating through spectral colors). Applied once when the evolution is gained; removed if the bolt is despawned.
4. No counter or gauge UI — the mechanic has no counter.

## What to Build

### 1. Generation-Depth Tracking (Shock Chain)

- Add generation depth metadata to shockwave VFX triggers. When a shockwave is attributed to Shock Chain, include the current generation depth.
- VFX function scales parameters by generation: HDR = 1.2 + (gen * 0.3), spark_count = 8 + (gen * 4).
- Chromatic aberration trigger per generation: intensity = gen * 0.1.
- Aggregate shake tracker: resource or system that counts Shock Chain kills in a rolling window (~0.5s) and triggers shake tier thresholds.

### 2. Cell Fission Animation (Split Decision)

- Fission animation system: on Split Decision trigger, animate the cell entity before despawn:
  - Cell glow splits along a random axis (two halves drift apart)
  - Halves condense into bolt orbs over ~0.15s
  - Energy filaments (thin beam entities) connect halves during split, fade on separation
- Flash at split: RadialBurst (10 sparks, HDR 0.8, short lifetime) + subtle screen flash.
- Prismatic birth trail: timed VisualModifier on spawned bolts (auto-remove after ~0.3s).
- Modifier inheritance: ensure spawned bolt entities copy parent's ModifierStack.

### 3. Triangle Charge Indicator (Circuit Breaker)

- Indicator entity system: three child entities arranged in equilateral triangle, parented to bolt entity.
  - Each node: small glow circle, starts at HDR ~0.2 (dim).
  - Thin connecting lines between nodes (also dim).
- Charge progression: on perfect bump with Circuit Breaker attribution, brighten next node to HDR ~1.0 with small flash particle.
- Circuit close: on 3rd bump:
  - All nodes flash to HDR >1.5
  - Nodes animate inward (collapse to center, ~0.1s)
  - Gold spark burst at center (20 sparks, HDR 1.5, gravity 40)
  - Enhanced shockwave (range 160, HDR 2.0, Gold)
  - Screen flash (Gold, intensity 0.8) + medium screen shake
- Reset: respawn indicator at dim state after payoff.
- Cleanup: despawn indicator when bolt is despawned (child entity relationship handles this).

### 4. Prismatic Flash System (Entropy Engine)

- Per-trigger flash: multi-colored RadialBurst (~12 particles, spectral color cycle, lifetime ~0.15s) at cell position. Fires before the selected effect's VFX.
- Persistent bolt shimmer: VisualModifier for color cycling (slow spectral rotation) applied to bolt while Entropy Engine is active.
- The selected random effect fires its own base VFX normally — Entropy Engine does not modify the downstream effect's visuals.

### 5. Shared: Evolution Spawn VFX Helpers

- Prismatic birth trail utility: reusable function to apply a timed prismatic color cycle modifier to newly spawned bolt entities. Used by both Split Decision and Supernova (5r).
- Spark burst at spawn position: reusable radial particle burst function used by multiple evolutions.

### 6. Tests

- Shock Chain shockwaves escalate in intensity per generation depth (HDR, spark count, chromatic aberration)
- Shock Chain aggregate shake fires at correct tier thresholds
- Split Decision plays fission animation on cell before despawn
- Split Decision spawned bolts receive prismatic birth trail modifier
- Split Decision spawned bolts inherit parent ModifierStack
- Circuit Breaker triangle indicator spawns anchored to bolt
- Circuit Breaker nodes brighten on perfect bump events
- Circuit Breaker payoff fires enhanced shockwave + screen effects on 3rd bump
- Circuit Breaker indicator resets to dim after payoff
- Circuit Breaker indicator despawns when bolt is despawned
- Entropy Engine prismatic flash fires before selected effect VFX
- Entropy Engine bolt shimmer modifier is active while evolution is active
- Attribution routing correctly identifies each evolution

## What NOT to Do

- Do NOT implement or modify the Shock Chain recursive kill mechanic. The VFX connects to whatever shockwaves the mechanic produces. If the recursive kill chain is not fully wired, the VFX still works for the shockwaves that do fire.
- Do NOT implement the Circuit Breaker counter mechanic. The `CircuitBreaker` effect leaf already tracks bumps internally. VFX hooks into the events it produces.
- Do NOT create a counter/gauge UI for Entropy Engine. The mechanic has no counter.
- Do NOT modify base shockwave, bolt-spawn, chain lightning, or speed boost VFX from 5l. Evolution VFX is additive or uses separate code paths.
- Do NOT create a recipe system. All VFX is direct Rust function calls.
- Do NOT make the Entropy Engine modify the selected random effect's VFX. The prismatic flash is a pre-effect; the selected effect fires its own base VFX.

## Dependencies

- **5l** (combat effect VFX): base Shockwave VFX (Shock Chain and Circuit Breaker enhance it), base ChainLightning and SpeedBoost VFX (Entropy Engine's random effects use them), base SpawnBolts VFX.
- **5c** (rantzsoft_particles2d): spark bursts, energy ring particles, glow mote particles for all evolutions.
- **5d** (rantzsoft_postprocess): chromatic aberration for Shock Chain, screen flash for Circuit Breaker and Split Decision.
- **5e** (visuals/ domain): VisualModifier for prismatic trail (Split Decision, Entropy Engine), ModifierStack for modifier inheritance (Split Decision), GlowParams for Circuit Breaker indicator nodes.
- **5k** (bump/failure VFX): screen shake infrastructure.
- **5q** (evo beams): evolution VFX routing mechanism (attribution detection).
- **5r** (evo AoE): Supernova's prismatic birth trail uses the same helper as Split Decision — shared utility created in whichever phase lands first.

## Verification

- Shock Chain shockwaves visibly escalate in brightness and particle density with each recursive generation
- Shock Chain chromatic aberration intensifies with depth
- Shock Chain screen shake scales with total chain size
- Split Decision shows a visible cell fission animation (cell splits, halves condense, filaments connect)
- Split Decision spawned bolts briefly show prismatic birth trails
- Split Decision spawned bolts visually match parent bolt (modifier inheritance)
- Circuit Breaker triangle indicator is visible near bolt and tracks bolt movement
- Circuit Breaker nodes light up progressively on perfect bumps
- Circuit Breaker payoff is dramatic (collapse, gold sparks, enhanced shockwave, screen flash, shake)
- Circuit Breaker indicator resets cleanly for the next cycle
- Entropy Engine shows prismatic flash on every cell destroy before the random effect fires
- Entropy Engine bolt has a persistent prismatic shimmer distinct from normal bolts
- All evolution VFX are visually a tier above base effects
- All existing tests pass

## Status: NEEDS DETAIL

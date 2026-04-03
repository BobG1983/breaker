# Phase 5 — Old to New Mapping

## Old Phase 5 Structure (21 sub-phases, monolithic rantzsoft_vfx)

| Old | Name | New fate |
|-----|------|----------|
| 5a | Rendering Architecture (docs) | DONE — design docs exist. Architecture doc rewritten in `phase-5-rethink/architecture.md` |
| 5b | Design Decisions (DR-1 through DR-10) | DONE — decisions still valid, not architecture-dependent |
| 5c | Crate setup + plugin separation | **REPLACED** — split into 5a (particles crate) + 5b (postprocess crate) + 5c (visuals domain) |
| 5d | Post-processing pipeline | **ABSORBED** into new 5b (postprocess crate) |
| 5e | Particle system | **ABSORBED** into new 5a (particles crate) |
| 5f | Temperature palette + data-driven enums | **SPLIT** — enums into new 5c (visuals domain), temperature into new 5h (dynamic visuals) |
| 5g | Bolt visuals | **KEPT** as new 5d |
| 5h | Breaker visuals | **KEPT** as new 5e |
| 5i | Cell visuals | **KEPT** as new 5f (merged with cell builder visual attachment) |
| 5j | Walls & background | **KEPT** as new 5g |
| 5k | Screen effects & feedback | **ABSORBED** into new 5h (dynamic visuals — screen shake, flash, distortion are postprocess triggers) |
| 5l | Bump grade & failure state VFX | **KEPT** as new 5i |
| 5m | Combat effect VFX | **KEPT** as new 5j |
| 5n | Visual modifier system | **ABSORBED** into new 5c (visuals domain — modifier types) + 5h (dynamic visuals — modifier computation) |
| 5o | Highlight moments | **KEPT** as new 5k |
| 5p | Transitions & PlayingState | **ABSORBED** into state lifecycle refactor (#1) — already happened |
| 5q | HUD & gameplay UI | **KEPT** as new 5l |
| 5r | Chip cards | **KEPT** as new 5m |
| 5s | Screens | **KEPT** as new 5n |
| 5t-5w | Evolution VFX batches 1-4 | **KEPT** as new 5o-5r |

## New Phase 5 Structure

### Infrastructure (no game visuals yet — just the tools)

| Phase | Name | What it delivers | Depends on |
|-------|------|-----------------|------------|
| **5c** | rantzsoft_particles2d crate | Particle engine, emitter, presets, plugin | Nothing |
| **5d** | rantzsoft_postprocess crate | FullscreenMaterial infra, 7 common shaders, trigger messages, plugin | Nothing |
| **5e** | visuals/ domain + entity shader | Hue, Shape, Aura, Trail, GlowParams, entity_glow SDF shader, additive material, VisualsPlugin, glitch_text shader, holographic shader | Nothing (types only, no rendering consumers yet) |

5c and 5d are independent — can run in parallel. 5e can also run in parallel (no crate dependency).

### Entity Visuals (making things look right)

| Phase | Name | What it delivers | Depends on |
|-------|------|-----------------|------------|
| **5f** | Bolt visuals | BoltDefinition rendering block in RON, bolt builder attaches Shape/Hue/Glow/Trail, wake/trail entities | 5e (visual types + entity shader) |
| **5g** | Breaker visuals | BreakerDefinition rendering block, breaker builder attaches Shape/Hue/Glow/Aura/Trail | 5e |
| **5h** | Cell visuals | CellTypeDefinition rendering block, cell builder attaches Shape/Hue/Glow, damage visual systems, death visual systems | 5e, cell builder (#4) |
| **5i** | Walls & background | Wall builder visual attachment, shield barrier shader, background grid shader, wall impact flash | 5e |

5f-5i are mostly independent (different domains) but all depend on 5e.

### Dynamic Visuals (runtime visual changes)

| Phase | Name | What it delivers | Depends on |
|-------|------|-----------------|------------|
| **5j** | Screen effects & modifiers | Screen shake, flash, desaturation, slow-mo (postprocess triggers from gameplay), ModifierStack + modifier computation system, temperature palette (RunTemperature + palette application), VisualModifier stacking with diminishing returns | 5d (postprocess crate), 5e (modifier types), 5f-5i (entities to modify) |

### Feedback & UI

| Phase | Name | What it delivers | Depends on |
|-------|------|-----------------|------------|
| **5k** | Bump grade & failure VFX | Bump flash/particles per grade, bolt lost VFX, life lost VFX, run end visual states | 5c (particles), 5d (screen flash), 5e (visual types) |
| **5l** | Combat effect VFX | Per-effect visual implementations (shockwave ring+distortion, chain lightning arcs, piercing beam, pulse, explode burst, gravity well lens, tether beam energy, etc.) | 5c (particles), 5d (distortion), 5e (visual types) |
| **5m** | Highlight moments | GlitchText overlay (Text2d + glitch_text shader), per-highlight game element VFX | 5e (glitch_text shader), 5k (screen flash) |
| **5n** | HUD & gameplay UI | Timer wall gauge shader, life orbs, node progress ticks | 5e (visual types) |
| **5o** | Chip cards | Entity composition, rarity treatments, holographic shader for evolution cards, abstract symbol icons | 5e (holographic shader, visual types) |
| **5p** | Screens | Main menu, run-end (victory splash + defeat hologram), breaker select, pause, loading | 5e, 5o (chip cards for selection screens) |

### Evolution VFX (crown jewels)

| Phase | Name | What it delivers | Depends on |
|-------|------|-----------------|------------|
| **5q** | Evolution VFX — beams | Nova Lance | 5l (combat VFX base) |
| **5r** | Evolution VFX — AoE | Supernova, Gravity Well, Dead Man's Hand | 5l |
| **5s** | Evolution VFX — chain/spawn | Chain Reaction, Split Decision, Feedback Loop, Entropy Engine | 5l |
| **5t** | Evolution VFX — entity effects | Phantom Breaker, Voltchain, ArcWelder, FlashStep, Second Wind | 5l |

## Parallelism

```
5c (particles) ──────────────────┐
5d (postprocess) ────────────────┤
5e (visuals domain + shaders) ───┤
                                 ├─→ 5f (bolt) ──┐
                                 ├─→ 5g (breaker)┤
                                 ├─→ 5h (cell) ──┤
                                 ├─→ 5i (walls) ─┤
                                 │                ├─→ 5j (screen fx + modifiers + temperature)
                                 │                │
                                 ├────────────────┼─→ 5k (bump/failure VFX)
                                 ├────────────────┼─→ 5l (combat effect VFX)
                                 │                │   │
                                 │                │   ├─→ 5q-5t (evolution VFX)
                                 │                │
                                 ├────────────────┼─→ 5m (highlights)
                                 ├────────────────┼─→ 5n (HUD)
                                 ├────────────────┼─→ 5o (chip cards)
                                 │                │   │
                                 │                │   ├─→ 5p (screens)
```

Key insight: 5c, 5d, 5e can all run in parallel (wave 1). Entity visuals (5f-5i) are wave 2. Everything else is wave 3+.

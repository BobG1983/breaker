# Phase 5 — Development Waves

## Wave 1: Infrastructure (no dependencies — all parallel)

| Phase | Name | Notes |
|-------|------|-------|
| **5a** | rantzsoft_particles2d crate | Standalone crate, no game code |
| **5b** | rantzsoft_postprocess crate | Standalone crate, no game code |
| **5c** | visuals/ domain + entity shader | Game-side types + shaders, no crate dependency |

All three are independent. Can run as 3 parallel branches.

## Wave 2: Entity Visuals (depends on 5c only — all parallel)

| Phase | Name | Depends on |
|-------|------|------------|
| **5d** | Bolt visuals | 5c |
| **5e** | Breaker visuals | 5c |
| **5f** | Cell visuals | 5c + cell builder (#4) |
| **5g** | Walls & background | 5c |

All four are independent of each other (different domains). Can run as 4 parallel branches after Wave 1.

Note: 5d-5g do NOT depend on 5a or 5b — they only attach visual components via builders. Particle/postprocess usage comes in later waves.

## Wave 3: Dynamic Visuals (depends on 5b + 5c + Wave 2)

| Phase | Name | Depends on |
|-------|------|------------|
| **5h** | Screen effects, modifiers & temperature | 5b, 5c, 5d-5g (entities must have materials to modify) |

Single phase. Bridges postprocess triggers into gameplay and activates the modifier computation system. Must wait for all entity visuals to land.

## Wave 4: Feedback & Combat VFX (depends on 5a + 5b + 5c + 5h)

| Phase | Name | Depends on |
|-------|------|------------|
| **5i** | Bump grade & failure VFX | 5a, 5b, 5c, 5d-5g, 5h |
| **5j** | Combat effect VFX | 5a, 5b, 5c, 5d-5g, 5h |

Both need particles (5a), postprocess (5b), visual types (5c), entity visuals (5d-5g), and screen effect lifecycle (5h). Independent of each other — can run in parallel.

## Wave 5: UI & Highlights (depends on 5c + 5h, some enhanced by Wave 4)

| Phase | Name | Depends on | Enhanced by |
|-------|------|------------|-------------|
| **5k** | Highlight moments | 5c, 5h, 5i | 5d, 5e, 5g |
| **5l** | HUD & gameplay UI | 5c, 5g, 5h | 5a, 5i, 5j |
| **5m** | Chip cards | 5c, 5h | 5a, 5k |

"Enhanced by" means the phase benefits from but doesn't strictly block on these — core functionality works without them, but visual polish is richer with them.

Can run in parallel. 5k and 5l can start as soon as Wave 3 finishes (5h). 5m can start as soon as 5c + 5h finish.

## Wave 6: Screens (depends on 5c + 5m)

| Phase | Name | Depends on | Enhanced by |
|-------|------|------------|-------------|
| **5n** | Screens | 5c, 5m | 5a, 5b, 5d, 5e, 5h, 5k, 5l |

Single phase. Depends on chip cards (5m) for selection screens and visual consistency. Many "enhanced by" dependencies but core screens work without them.

## Wave 7: Evolution VFX (depends on 5j — all parallel)

| Phase | Name | Depends on |
|-------|------|------------|
| **5o** | Evolution VFX — beams | 5j |
| **5p** | Evolution VFX — AoE | 5j |
| **5q** | Evolution VFX — chain/spawn | 5j |
| **5r** | Evolution VFX — entity effects | 5j |

All four are independent (different evolutions). Can run as 4 parallel branches after combat VFX (5j) lands.

## Summary

| Wave | Phases | Parallelism | Blocked by |
|------|--------|-------------|------------|
| 1 | 5a, 5b, 5c | 3 parallel | Nothing |
| 2 | 5d, 5e, 5f, 5g | 4 parallel | Wave 1 (5c only) |
| 3 | 5h | 1 serial | Wave 1 (5b) + Wave 2 |
| 4 | 5i, 5j | 2 parallel | Wave 1 (5a, 5b) + Wave 3 |
| 5 | 5k, 5l, 5m | 3 parallel | Wave 3 (5h) |
| 6 | 5n | 1 serial | Wave 5 (5m) |
| 7 | 5o, 5p, 5q, 5r | 4 parallel | Wave 4 (5j) |

Total: 18 phases across 7 waves. Maximum parallelism: 4 (Waves 2 and 7).

## Flow Diagram

```
                    WAVE 1 (infrastructure — all parallel)
            ┌──────────────┬──────────────┬──────────────┐
            │              │              │              │
          [5a]           [5b]           [5c]             │
       particles      postprocess   visuals/domain       │
            │              │              │              │
            │              │     ┌────────┴────────┐     │
            │              │     │   WAVE 2         │     │
            │              │     │ (entity visuals) │     │
            │              │     │   all parallel   │     │
            │              │  ┌──┴──┬──┬──┬──┐      │     │
            │              │  │[5d] │[5e]│[5f]│[5g]│      │
            │              │  │bolt │brkr│cell│wall│      │
            │              │  └──┬──┴──┴──┴──┘      │     │
            │              │     │                   │     │
            │              │     ▼                   │     │
            │              └──►[5h]◄─────────────────┘     │
            │              WAVE 3 (dynamic visuals)        │
            │              screen fx + modifiers + temp     │
            │                    │                          │
            │         ┌──────────┤                          │
            │         │          │                          │
            │         ▼          ▼                          │
            └──────►[5i]       [5j]◄────────────────────────┘
                  WAVE 4 (feedback + combat — parallel)
                  bump/fail    combat VFX
                    │            │
           ┌────────┤            │
           │        │            │
           ▼        ▼            │
         [5k]     [5l]   [5m]   │
        WAVE 5 (UI + highlights — parallel)
        highlights  HUD   cards  │
                          │      │
                          ▼      │
                        [5n]     │
                       WAVE 6    │
                       screens   │
                                 │
                    ┌────────────┴────────────┐
                    │      WAVE 7              │
                    │ (evolution VFX — parallel)│
                  ┌─┴──┬──┬──┬──┐             │
                  │[5o]│[5p]│[5q]│[5r]│             │
                  │beam│AoE │chn │ent │             │
                  └────┴────┴────┴────┘             │
```

# Phase 2: Game Loop & Time Pressure

**Goal**: Turn the sandbox into a game with stakes. A 3-node run with a timer, breaker archetypes, node transitions through a placeholder upgrade screen, and game-over on timer expiry.

## Subphases

- [2a: Level Loading](phase-2a-level-loading.md) — Hand-authored RON layouts, node completion detection
- [2b: Run Structure & Node Timer](phase-2b-run-and-timer.md) — 3-node sequential runs, countdown timer, game-over, run-end screen
- [2c: Archetype System & Aegis](phase-2c-archetype-system.md) — Polymorphic bolt-lost dispatch, per-breaker RON config, Aegis as proof-of-concept
- [2d: Screens](phase-2d-screens-and-ui.md) — Breaker selection, placeholder upgrade screen, pause menu
- [2e: Visual Polish & Additional Archetypes](phase-2e-chrono-and-prism.md) — Visual interpolation, Chrono, Prism

## Build Order Rationale

2a-2b delivers a playable timed 3-node loop with a single breaker. 2c builds the archetype *system* and proves it works with Aegis (simplest bolt-lost behavior). 2d adds screens — informed by having a real archetype to display in selection UI. 2e fills in Chrono and Prism using the proven system.

Archetypes come before screens because breaker selection UI should be designed with concrete archetypes in hand, not stubs.

## Design Decisions (from planning interview)

- **Bump multipliers**: All grades boost — early/late = small boost, perfect = big boost, no bump = neutral (1.0x). Attempting the mechanic is always rewarded.
- **Breaker finality**: Aegis/Chrono/Prism are proof-of-concept designs to validate the archetype system. Shipped breakers may differ.
- **Archetype depth**: Broader differentiation — bolt-lost + different base stats + different dash/bump properties. Upgrade affinities noted for Phase 6+.
- **Seeded determinism**: Deferred to Phase 4 (vertical slice). Accept some retrofit cost.

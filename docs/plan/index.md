# Brickbreaker Roguelite — Build Plan

## Context
Building a roguelite Arkanoid clone in Bevy (Rust). Solo dev, architecture-first approach validated through a vertical slice. Stylized shader-driven visuals, custom physics, hybrid data model (mechanics in code, content in data files), audio planned from the start.

See `../design/` for core design principles and decisions, `../architecture/` for technical decisions, and `../design/terminology/` for game vocabulary.

## Phases

### Done
- [Phase 0: Project Scaffolding & Architecture](done/phase-0-scaffolding.md)
- [Phase 1: Core Mechanics](done/phase-1/index.md)
  - [1a: Breaker System](done/phase-1/phase-1a-breaker.md)
  - [1b: Bolt](done/phase-1/phase-1b-bolt.md)
  - [1c: Cells](done/phase-1/phase-1c-cells.md)
  - [1d: Bump & Perfect Bump](done/phase-1/phase-1d-bump.md)
- [Phase 2: Game Loop & Time Pressure](done/phase-2/index.md)
  - [2a: Level Loading](done/phase-2/phase-2a-level-loading.md)
  - [2b: Run Structure & Node Timer](done/phase-2/phase-2b-run-and-timer.md)
  - [2c: Archetype System & Aegis](done/phase-2/phase-2c-archetype-system.md)
  - [2d: Screens](done/phase-2/phase-2d-screens-and-ui.md)
  - [2e: Visual Polish & Additional Archetypes](done/phase-2/phase-2e-chrono-and-prism.md)
- [Phase 3: Dev Infrastructure](done/phase-3/index.md)
  - [3a: Workspace Restructure](done/phase-3/phase-3a-workspace-restructure.md)
  - [3b: Debug Domain Restructure](done/phase-3/phase-3b-debug-restructure.md)
  - [3c: RON Hot-Reload](done/phase-3/phase-3c-hot-reload.md)
  - [3d: Scenario Runner](done/phase-3/phase-3d-scenario-runner.md)
  - [3e: Structured Logging](done/phase-3/phase-3e-structured-logging.md)
- [Phase 4: Vertical Slice — Mini-Run](done/phase-4/index.md)
  - [4a: Seeded RNG & Run Seed](done/phase-4/phase-4a-seeded-rng.md)
  - [4b: Chip Effect System](done/phase-4/phase-4b-chip-effects.md)
  - [4c: Chip Pool & Rarity](done/phase-4/phase-4c-chip-pool.md)
  - [4d: Trigger/Effect Architecture](done/phase-4/phase-4d-trigger-effect.md)
  - [4e: Node Sequence & Escalation](done/phase-4/phase-4e-node-escalation.md)
  - [4f: Chip Offering System](done/phase-4/phase-4f-chip-offerings.md)
  - [4g: Node Transitions & VFX](done/phase-4/phase-4g-node-transitions.md)
  - [4h: Chip Evolution](done/phase-4/phase-4h-chip-evolution.md)
  - [4i: Run Stats & Summary](done/phase-4/phase-4i-run-stats.md)
  - [4j: Release Infrastructure](done/phase-4/phase-4j-release-infrastructure.md)
  - Post-Wave: Spatial/Physics Extraction — Done
  - Post-Wave: Stat Effects — Done
  - Post-Wave: Runtime Effects — Done

### In Progress
- ~~Graphics & Sound Audit~~ — Done. Catalog at `docs/design/graphics/catalog/`
- [Phase 5: Visual Identity](phase-5/index.md) — 23 steps: architecture, rendering/ domain, post-processing, particles, entity visuals, effects, UI, evolution VFX
  - [5a: Rendering Architecture](phase-5/phase-5a-rendering-architecture.md) — `docs/architecture/rendering.md`
  - [5b: Design Decisions](phase-5/phase-5b-design-decisions.md) — **DECISION REQUIRED**
  - [5c: Render Plugin Separation](phase-5/phase-5c-render-plugin-separation.md)
  - [5d: Post-Processing Pipeline](phase-5/phase-5d-post-processing-pipeline.md)
  - [5e: Particle System](phase-5/phase-5e-particle-system.md)
  - [5f: Temperature Palette & Data-Driven Enums](phase-5/phase-5f-temperature-and-enums.md)
  - [5g: Bolt Visuals](phase-5/phase-5g-bolt-visuals.md)
  - [5h: Breaker Visuals](phase-5/phase-5h-breaker-visuals.md)
  - [5i: Cell Visuals](phase-5/phase-5i-cell-visuals.md)
  - [5j: Walls & Background](phase-5/phase-5j-walls-and-background.md)
  - [5k: Screen Effects & Feedback](phase-5/phase-5k-screen-effects.md)
  - [5l: Bump Grade & Failure State VFX](phase-5/phase-5l-bump-and-failure-vfx.md)
  - [5m: Combat Effect VFX](phase-5/phase-5m-combat-effect-vfx.md)
  - [5n: Visual Modifier System](phase-5/phase-5n-visual-modifiers.md)
  - [5o: Highlight Moments](phase-5/phase-5o-highlight-moments.md)
  - [5p: Transitions](phase-5/phase-5p-transitions.md)
  - [5q: HUD & Gameplay UI](phase-5/phase-5q-hud-ui.md) — Diegetic HUD
  - [5r: Chip Cards](phase-5/phase-5r-chip-cards.md) — Abstract symbol icons
  - [5s: Screens](phase-5/phase-5s-screens.md) — Hybrid run-end
  - [5t: Evolution VFX Batch 1 — Beams](phase-5/phase-5t-evo-beams.md) — Nova Lance
  - [5u: Evolution VFX Batch 2 — AoE](phase-5/phase-5u-evo-aoe.md)
  - [5v: Evolution VFX Batch 3 — Chain/Spawn](phase-5/phase-5v-evo-chain-spawn.md)
  - [5w: Evolution VFX Batch 4 — Entity Effects](phase-5/phase-5w-evo-entity-effects.md)

### Upcoming
- [Phase 6: Audio Foundation](phase-6-audio.md)
- [Phase 7: Content & Variety](phase-7-content.md)
- [Phase 8: Roguelite Progression](phase-8-roguelite.md)
- [Phase 9: Boss Nodes & Advanced Mechanics](phase-9-bosses.md)
- [Phase 10: Social Shareability](phase-10-social-shareability.md) — Video clips of highlight moments, run-end playback, social media sharing
- [Phase 11: Polish & Ship](phase-11-polish.md)
- **Asset Validator** (future) — Standalone tool that loads all RON assets, validates deserialization, checks recipe ingredient references, effect variant existence, and cross-file data consistency. Complements scenario runner (which tests gameplay) with static data validation.

## Build Order Rationale

The plan front-loads **feel** (Phases 1-2) because if the breaker-bolt-cell interaction isn't satisfying, nothing else matters. **Dev infrastructure** (Phase 3) comes next — hot-reload and scenario testing make the vertical slice faster to iterate on. The **vertical slice** (Phase 4) validates architecture under real gameplay conditions before investing in content. Visuals and audio (Phases 5-6) come before content expansion because the stylized aesthetic is part of the game's identity and informs how content is designed. Roguelite systems (Phase 8) are deliberately late — they're important for retention but meaningless without solid core gameplay.

## Resolved Design Decisions

Documented in full at `../design/decisions/`. Summary:

| Decision | Summary | Detail |
|----------|---------|--------|
| Chip stacking | Hybrid: per-chip caps, flat integer stacks, Isaac-style pool depletion | [chip-stacking.md](../design/decisions/chip-stacking.md) |
| Chip evolution | Two chips at min stacks + boss kill = evolved chip | [chip-evolution.md](../design/decisions/chip-evolution.md) |
| Chip offerings | Weighted random, rarity tiers, weight decay on seen chips | [chip-offering-system.md](../design/decisions/chip-offering-system.md) |
| Chip timeout | Timer expires = skip (no chip). Maximum pressure | [chip-timeout.md](../design/decisions/chip-timeout.md) |
| Bump multipliers | All grades boost — early/late = small, perfect = large, no bump = neutral | [bump-multipliers.md](../design/decisions/bump-multipliers.md) |
| Node escalation | Procedural tiers from seed, passive -> active -> boss cadence | [node-escalation.md](../design/decisions/node-escalation.md) |
| Seeded determinism | Phase 4. Run seed drives all randomness | [seeded-determinism.md](../design/decisions/seeded-determinism.md) |
| Chip synergies | 30%+ chips must interact with other chips' effects. Web, not list | [chip-synergies.md](../design/decisions/chip-synergies.md) |
| Chip templates | One RON file per chip concept with per-rarity slots, shared max_taken | [chip-template-system.md](../design/decisions/chip-template-system.md) |
| Chip rarity | Design Rare first, derive weaker Common/Uncommon; Legendaries are niche max:1 | [chip-rarity-rework.md](../design/decisions/chip-rarity-rework.md) |
| Breaker archetypes | Proof-of-concept designs, system flexibility over final balance | [breaker-archetypes.md](../design/decisions/breaker-archetypes.md) |
| Workspace structure | Axum-style — `breaker-game/`, `rantzsoft_spatial2d/`, `rantzsoft_physics2d/`, `rantzsoft_defaults/`, `rantzsoft_defaults_derive/`, `breaker-scenario-runner/` as peer directories |
| Scenario runner | Separate crate with argh CLI. Visual + headless modes. RON scenario files crate-local |

## Open Questions
1. **Portal cell scope**: Sub-levels are a game-within-a-game. How complex? Defer to Phase 9?

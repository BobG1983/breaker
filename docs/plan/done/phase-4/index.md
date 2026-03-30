# Phase 4: Vertical Slice — Mini-Run

> **Phase 4 is complete. All stages shipped.**

**Goal**: A playable multi-tier run that proves the architecture and feels like the game. Seeded determinism, functional chip effects with stacking and evolution, weighted offering system with pool depletion, procedural node escalation with boss nodes, animated transitions, and a full run summary.

## Stages

| Stage | Name | Depends On | Size | Status |
|-------|------|-----------|------|--------|
| 4a | Seeded RNG & Run Seed | — | Small | DONE |
| [4b](phase-4b-chip-effects.md) | Chip Effect System | — | Medium-Large | DONE |
| [4c](phase-4c-chip-pool.md) | Chip Pool & Rarity | 4b | Large | DONE |
| [4d](phase-4d-trigger-effect.md) | Trigger/Effect Architecture | 4b | Large | DONE |
| [4e](phase-4e-node-escalation.md) | Node Sequence & Escalation | 4a | Very Large | DONE |
| [4f](phase-4f-chip-offerings.md) | Chip Offering System | 4a, 4c | Medium | DONE |
| [4g](phase-4g-node-transitions.md) | Node Transitions & VFX | 4e | Medium | DONE |
| [4h](phase-4h-chip-evolution.md) | Chip Evolution | 4c, 4d, 4e | Medium | DONE |
| [4i](phase-4i-run-stats.md) | Run Stats & Summary | 4e, 4f | Medium | DONE |
| [4j](phase-4j-release-infrastructure.md) | Release Infrastructure | — | Small | DONE |
| Post-Wave | Spatial/Physics Extraction | Phase 4 complete | — | DONE |
| Post-Wave | Stat Effects | Phase 4 complete | — | DONE |
| Post-Wave | Runtime Effects | Phase 4 complete | — | DONE |

## Dependency Graph

```
4a (Seeded RNG) ──────┬──── 4e (Node Escalation) ──┬── 4g (Transitions)
                      │                             │
                      └──── 4f (Chip Offerings) ────┤
                                                    │
4b (Chip Effects) ──┬── 4c (Chip Pool) ─────────────┤── 4h (Evolution)
                    │                               │
                    └── 4d (Trigger/Effect) ────────┘
                                                    │
                                                    └── 4i (Run Stats)

4j (Release Infrastructure) ── no gameplay dependencies, runs in Wave 4
```

## Critical Paths

Two chains determined overall Phase 4 duration:

1. **Chip chain**: 4b -> 4d -> 4h — included the riskiest stage (4d, architecturally novel trigger chains). Starting 4b in Wave 1 was essential.
2. **Node chain**: 4a -> 4e -> 4g — included the largest single stage (4e). Unblocking it immediately with 4a was important.

## Implementation Waves

### Wave 1 — Foundations (parallel, no dependencies)

Started **4a** and **4b** simultaneously. They were completely independent.

- **4a (Seeded RNG)** — small, self-contained. Single delegated pair (writer-tests -> writer-code). Main agent handled RunSeed resource wiring in shared/.
- **4b (Chip Effect System)** — too large for one shot. Split into sub-stages:
  - **4b.1**: Effect type definitions (AmpEffect, AugmentEffect enums) + ChipSelected handler + stacking logic — all within chips/ domain
  - **4b.2**: Per-domain effect consumption — each touched a different domain and parallelized:
    - Piercing -> physics/ (bolt_cell_collision)
    - DamageBoost -> cells/ (handle_cell_hit)
    - SpeedBoost -> bolt/ (prepare_bolt_velocity)
    - WidthBoost -> breaker/ (spawn/init)

**Session 1**: 4a + 4b.1 in parallel — DONE
**Session 2**: 4b.2 (all effect implementations across domains in parallel) — DONE

### Wave 2 — Core Systems (after Wave 1, partially parallel)

Three stages unlocked. **4c**, **4d**, and **4e** all parallelized since they depended on different Wave 1 outputs and touched different domains.

- **4e (Node Escalation)** — the biggest stage, broken down:
  - **4e.1**: Tier data structures + difficulty curve RON format (run/ domain)
  - **4e.2**: Procedural node sequence generation algorithm (run/ domain, pure logic)
  - **4e.3**: New cell types — each independent, parallelized:
    - Lock cells -> cells/ domain
    - Regen cells -> cells/ domain
  - **4e.4**: Layout pool reorganization (assets/nodes/passive/, active/, boss/) + loader updates

- **4c (Chip Pool & Rarity)** — Split into:
  - **4c.1**: Rarity enum + ChipInventory resource + max_stacks tracking (chips/ domain)
  - **4c.2**: Author 16-20 chip RON files (content, batched)
  - **4c.3**: Synergy design review (guard-game-design validation)

- **4d (Trigger/Effect Architecture)** — Split into:
  - **4d.1**: TriggerChain enum + RON parsing (in chips/definition.rs)
  - **4d.2**: Unified evaluation engine (behaviors/evaluate.rs, behaviors/active.rs, behaviors/armed.rs, behaviors/events.rs)
  - **4d.3**: Shockwave effect in behaviors/effects/shockwave.rs
  - **4d.4**: Surge overclock end-to-end via surge_overclock.scenario.ron with initial_overclocks field

**Session 3**: 4e.1 + 4e.2 + 4c.1 in parallel — DONE
**Session 4**: 4e.3 (Lock + Regen cells in parallel) + 4e.4 (layout pools) — DONE
**Session 5**: 4d.1 + 4d.2 (trigger types + unified evaluation engine) — DONE
**Session 6**: 4d.3 + 4d.4 (shockwave + Surge POC; 4c.2 chip RON authoring deferred) — DONE

### Wave 3 — Integration (after Wave 2, parallel)

- **4f (Chip Offerings)** (needed 4a + 4c) — weighted random selection, pool depletion. Touched chips/ domain.
- **4g (Node Transitions)** (needed 4e) — VFX, state machine extension. Touched screen/fx domains.

These were independent and parallelized.

**Session 7**: 4f + 4g in parallel — DONE

### Wave 4 — Capstones (after Wave 3, parallel)

- **4h (Chip Evolution)** (needed 4c + 4d + 4e) — evolution recipes, boss reward screen, evolution registry.
- **4i (Run Stats)** (needed 4e + 4f) — purely observational systems, no mutation.
- **4j (Release Infrastructure)** (no gameplay dependencies) — GitHub Actions cross-compilation, itch.io butler, version bumping, changelog. Used the **runner-release** agent.

All three were independent and parallelized.

**Session 8**: 4h + 4i + 4j in parallel — DONE

### Post-Wave — Render Separation (feeds into Phase 5)

- **4k (Render Plugin Separation)** (no gameplay dependencies) — Extract all visual-only concerns from gameplay plugins into a centralized render domain. Currently, spawn systems (bolt, breaker, cells) mix gameplay components with visual components (`Mesh2d`, `MeshMaterial2d`, `TextFont`), and screen spawn systems are entirely visual. This refactor would:
  - Split gameplay spawn systems into "spawn entity" + "attach visuals" (via `Added<T>` observers)
  - Move screen spawn systems (`spawn_main_menu`, `spawn_timer_hud`, etc.) into a render plugin
  - Move feedback systems (`bump_feedback`, `bolt_lost_feedback`) into a render plugin
  - Eliminate `HeadlessAssetsPlugin` — headless mode simply omits the render plugin
  - Enable future alternative visual modes (debug wireframe, replay viewer)
  - **Trigger**: Do this when Phase 5 (Visual Identity) adds enough new visual systems that the separation pays for itself. Not worth doing before then.
  - **Scope**: ~14 spawn/feedback systems to split, plus screen plugin restructuring.

### Post-Wave — Graphics & Sound Audit (feeds into Phases 5 + 6)

- **4L (Graphics & Sound Audit)** — Comprehensive audit of every game entity, chip effect, breaker behavior, cell type, and triggered effect to catalog what graphics and sound each one needs. This audit produces the work list for Phase 5 (Visual Identity) and Phase 6 (Audio). Scope:
  - **Breakers**: Each breaker archetype (Aegis, Chrono, Prism) — visual identity, bump feedback VFX, dash/settle/brake animations, bump grade flash effects, sound effects for bump grades
  - **Bolt**: Base bolt, extra bolts, flail bolts — motion trails, impact flashes, speed-dependent visual intensity, wall/cell/breaker impact sounds
  - **Cells**: Per-cell-type visual identity (Standard, Lock, Regen, Shield/Orbit, Armored, Explosive) — idle animations, damage states, destruction VFX (dissolve/shatter/explode), hit sounds, destruction sounds
  - **Chip effects**: Every `Effect` enum variant — triggered VFX (shockwave rings, chain lightning arcs, piercing beams, gravity well distortion, phantom breaker ghost), activation sounds, stacking visual feedback
  - **Chip selection**: Rarity-based card glow, selection confirmation VFX, evolution unlock fanfare
  - **Node transitions**: Flash/sweep animations (already partially implemented), level-complete/level-start sounds
  - **UI**: Timer urgency escalation (visual + audio), score display, chip HUD icons, run summary screen
  - **Background**: Animated backdrop responding to game state intensity
  - **Screen effects**: Screen shake, chromatic aberration, vignette
  - **Multi-shockwave readability**: Color shift per source (hue rotation) so overlapping shockwaves are distinguishable
  - **Flail tether visual**: Chain/tether rendering between flail bolt pairs (segmented line mesh, catenary curve, neon glow)
  - **Shield unlock shatter**: Fragment VFX on shield cell unlock
  - **Trigger**: After Phase 4 gameplay is complete. The audit itself is design work, not code.
  - **Output**: Work list feeding Phase 5 (visual) and Phase 6 (audio) implementation plans.

## Session Summary

| Session | Stages | Focus | Domains Touched | Status |
|---------|--------|-------|-----------------|--------|
| **1** | 4a + 4b.1 | Seeded RNG + chip effect types/handler/stacking | shared, chips | DONE |
| **2** | 4b.2 | Per-domain effect consumption (parallel across domains) | physics, cells, bolt, breaker | DONE |
| **3** | 4e.1 + 4e.2 + 4c.1 | Tier structures + proc-gen algorithm + rarity/inventory | run, chips | DONE |
| **4** | 4e.3 + 4e.4 | New cell types (Lock, Regen) + layout pool reorg | cells, assets | DONE |
| **5** | 4d.1 + 4d.2 | TriggerChain types + unified chain evaluation engine | behaviors, chips | DONE |
| **6** | 4d.3 + 4d.4 | Shockwave + Surge POC (4c.2 deferred) | behaviors, assets | DONE |
| **7** | 4f + 4g | Chip offerings + node transitions (parallel) | chips, screen, fx | DONE |
| **8** | 4h + 4i + 4j | Evolution + run stats + release infra (parallel capstones) | chips, run, ui, CI | DONE |

## Scenario Coverage

Each stage included a **Scenario Coverage** section listing suggested invariants and scenario RON files. These were starting points, not exhaustive lists. If implementation revealed new properties that should always hold, new edge cases worth stress-testing, or new chaos-input failure modes — invariants and scenarios were added as needed.

## Design Decisions

All design decisions for Phase 4 are documented in `../../design/decisions/`:
- [Chip Stacking](../../design/decisions/chip-stacking.md)
- [Chip Evolution](../../design/decisions/chip-evolution.md)
- [Chip Offering System](../../design/decisions/chip-offering-system.md)
- [Chip Selection Timeout](../../design/decisions/chip-timeout.md)
- [Node Escalation](../../design/decisions/node-escalation.md)
- [Seeded Determinism](../../design/decisions/seeded-determinism.md)
- [Chip Synergies](../../design/decisions/chip-synergies.md)

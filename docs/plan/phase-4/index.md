# Phase 4: Vertical Slice — Mini-Run

**Goal**: A playable multi-tier run that proves the architecture and feels like the game. Seeded determinism, functional chip effects with stacking and evolution, weighted offering system with pool depletion, procedural node escalation with boss nodes, animated transitions, and a full run summary.

## Stages

| Stage | Name | Depends On | Size | Risk |
|-------|------|-----------|------|------|
| [4a](phase-4a-seeded-rng.md) | Seeded RNG & Run Seed | — | Small | Low |
| [4b](phase-4b-chip-effects.md) | Chip Effect System | — | Medium-Large | Medium |
| [4c](phase-4c-chip-pool.md) | Chip Pool & Rarity | 4b | Large (content-heavy) | Low technical, high design |
| [4d](phase-4d-trigger-effect.md) | Trigger/Effect Architecture | 4b | Large | **High** — new pattern |
| [4e](phase-4e-node-escalation.md) | Node Sequence & Escalation | 4a | Very Large | Medium |
| [4f](phase-4f-chip-offerings.md) | Chip Offering System | 4a, 4c | Medium | Low |
| [4g](phase-4g-node-transitions.md) | Node Transitions & VFX | 4e | Medium | Low |
| [4h](phase-4h-chip-evolution.md) | Chip Evolution | 4c, 4d, 4e | Medium | Low |
| [4i](phase-4i-run-stats.md) | Run Stats & Summary | 4e, 4f | Medium | Low |
| [4j](phase-4j-release-infrastructure.md) | Release Infrastructure | — | Small | Low |

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

Two chains determine overall Phase 4 duration:

1. **Chip chain**: 4b → 4d → 4h — includes the riskiest stage (4d, architecturally novel trigger chains). Starting 4b in Wave 1 is essential.
2. **Node chain**: 4a → 4e → 4g — includes the largest single stage (4e). Unblocking it immediately with 4a is important.

## Implementation Waves

### Wave 1 — Foundations (parallel, no dependencies)

Start **4a** and **4b** simultaneously. They are completely independent.

- **4a (Seeded RNG)** — small, self-contained. Single delegated pair (writer-tests → writer-code). Main agent handles RunSeed resource wiring in shared/.
- **4b (Chip Effect System)** — too large for one shot. Split into sub-stages:
  - **4b.1**: Effect type definitions (AmpEffect, AugmentEffect enums) + ChipSelected handler + stacking logic — all within chips/ domain
  - **4b.2**: Per-domain effect consumption — each touches a different domain and can parallelize:
    - Piercing → physics/ (bolt_cell_collision)
    - DamageBoost → cells/ (handle_cell_hit)
    - SpeedBoost → bolt/ (prepare_bolt_velocity)
    - WidthBoost → breaker/ (spawn/init)

**Session 1**: 4a + 4b.1 in parallel
**Session 2**: 4b.2 (all effect implementations across domains in parallel)

### Wave 2 — Core Systems (after Wave 1, partially parallel)

Three stages unlock. **4c**, **4d**, and **4e** can all parallelize since they depend on different Wave 1 outputs and touch different domains.

- **4e (Node Escalation)** — the biggest stage, must be broken down:
  - **4e.1**: Tier data structures + difficulty curve RON format (run/ domain)
  - **4e.2**: Procedural node sequence generation algorithm (run/ domain, pure logic)
  - **4e.3**: New cell types — each is independent, can parallelize:
    - Lock cells → cells/ domain
    - Regen cells → cells/ domain
  - **4e.4**: Layout pool reorganization (assets/nodes/passive/, active/, boss/) + loader updates

- **4c (Chip Pool & Rarity)** — large, split into:
  - **4c.1**: Rarity enum + ChipInventory resource + max_stacks tracking (chips/ domain)
  - **4c.2**: Author 16-20 chip RON files (content, can batch)
  - **4c.3**: Synergy design review (guard-game-design validation)

- **4d (Trigger/Effect Architecture)** — riskiest stage, split into:
  - **Research**: Use researcher-bevy-api to verify observer/event pattern for trigger chains
  - **4d.1**: TriggerChain enum + RON parsing (types only)
  - **4d.2**: Bolt behaviors module + intermediate state markers (bolt/ domain)
  - **4d.3**: Shockwave effect implementation
  - **4d.4**: Surge overclock end-to-end proof-of-concept (integration, likely manual)

**Session 3**: 4e.1 + 4e.2 + 4c.1 in parallel (data structures + algorithm + rarity system)
**Session 4**: 4e.3 (Lock + Regen cells in parallel) + 4e.4 (layout pools)
**Session 5**: 4d.1 + 4d.2 (trigger types + bolt behaviors — after researcher-bevy-api)
**Session 6**: 4d.3 + 4d.4 + 4c.2 (shockwave + Surge POC + chip RON authoring)

### Wave 3 — Integration (after Wave 2, parallel)

- **4f (Chip Offerings)** (needs 4a + 4c) — weighted random selection, pool depletion. Touches chips/ domain.
- **4g (Node Transitions)** (needs 4e) — VFX, state machine extension. Touches screen/fx domains.

These are independent and can parallelize.

**Session 7**: 4f + 4g in parallel

### Wave 4 — Capstones (after Wave 3, parallel)

- **4h (Chip Evolution)** (needs 4c + 4d + 4e) — evolution recipes, boss reward screen, evolution registry.
- **4i (Run Stats)** (needs 4e + 4f) — purely observational systems, no mutation.
- **4j (Release Infrastructure)** (no gameplay dependencies) — GitHub Actions cross-compilation, itch.io butler, version bumping, changelog. Uses the **runner-release** agent.

All three are independent and can parallelize.

**Session 8**: 4h + 4i + 4j in parallel

## Session Summary

| Session | Stages | Focus | Domains Touched |
|---------|--------|-------|-----------------|
| **1** | 4a + 4b.1 | Seeded RNG + chip effect types/handler/stacking | shared, chips |
| **2** | 4b.2 | Per-domain effect consumption (parallel across domains) | physics, cells, bolt, breaker |
| **3** | 4e.1 + 4e.2 + 4c.1 | Tier structures + proc-gen algorithm + rarity/inventory | run, chips |
| **4** | 4e.3 + 4e.4 | New cell types (Lock, Regen) + layout pool reorg | cells, assets |
| **5** | 4d.1 + 4d.2 | TriggerChain types + bolt behaviors module | bolt, chips |
| **6** | 4d.3 + 4d.4 + 4c.2 | Shockwave + Surge POC + chip RON authoring | bolt, physics, assets |
| **7** | 4f + 4g | Chip offerings + node transitions (parallel) | chips, screen, fx |
| **8** | 4h + 4i + 4j | Evolution + run stats + release infra (parallel capstones) | chips, run, ui, CI |

## What NOT to Combine

- **4d + 4e** — both large, different domains, different risk profiles
- **All of Wave 2 at once** — 4c + 4d + 4e together exceeds reasonable context. Start with structural pieces, then implement logic.
- **4b as a monolith** — touches 4+ domains, must split effect types from effect consumption.

## Scenario Coverage

Each stage includes a **Scenario Coverage** section listing suggested invariants and scenario RON files. These are starting points, not exhaustive lists. If implementation reveals new properties that should always hold, new edge cases worth stress-testing, or new chaos-input failure modes — add invariants and scenarios as needed. The suggestions exist to prompt thinking about runtime validation, not to cap it.

## Cross-Cutting Work (Main Agent Only)

These tasks span multiple domains and cannot be delegated:
- Shared type creation (new message types, shared enums used by multiple domains)
- `lib.rs` / `game.rs` / `shared.rs` wiring for new plugins or modules
- New domain creation wiring (if bolt/behaviors becomes a new sub-domain)
- Architectural decisions for 4d's trigger chain pattern

## Design Decisions

All design decisions for Phase 4 are documented in `../../design/decisions/`:
- [Chip Stacking](../../design/decisions/chip-stacking.md)
- [Chip Evolution](../../design/decisions/chip-evolution.md)
- [Chip Offering System](../../design/decisions/chip-offering-system.md)
- [Chip Selection Timeout](../../design/decisions/chip-timeout.md)
- [Node Escalation](../../design/decisions/node-escalation.md)
- [Seeded Determinism](../../design/decisions/seeded-determinism.md)
- [Chip Synergies](../../design/decisions/chip-synergies.md)

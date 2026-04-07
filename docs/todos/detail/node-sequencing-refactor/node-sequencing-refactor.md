# Node Sequencing Refactor

## Summary
Replace full upfront node sequence generation with per-tier batching. Tiers define difficulty progression and node type escalation across a run.

## Context
Currently all nodes for a run are generated upfront. This needs to change to a Balatro-inspired tier system where each tier is generated when the player reaches it, allowing difficulty to scale dynamically and enabling mechanics like tier regression.

## Sub-Documents
- [Tier Structure & Progression](tiers.md) — tier structure, node type ramp, tier 0, infinite scaling
- [Cell Types](cell-types.md) — 7 cell types + portal cells, introduction timeline, combos
- [Node Generation](node-generation.md) — three-tier composition system (frames/blocks/skeletons), boss nodes
- [Protocol & Hazard system](../mod-system-design.md) — separate todo

## What's Decided
- Tier structure (4+boss, 8 tiers, infinite continuation)
- Node type ramp (PPPA through VVVV)
- Cell types and introduction timeline (7 types, tiers 1-7)
- Portal cell rules (required, zero nesting, tier-2 difficulty, cap 4)
- Node generation approach (frames > blocks > skeletons)
- Boss nodes are fully hand-designed with code-driven brains

## What Needs Detail
- **RON formats**: frame, block, and skeleton RON file formats — what do they actually look like?
- **Skeleton ↔ modifier integration**: how does skeleton format interact with cell modifier RON format? How does resolution feed into `Cell::builder()`?
- **Block subdivision**: how are frame slot areas divided into blocks? Predefined splits or algorithmic?
- **Seed propagation**: deterministic seed flow through frame → block → skeleton → modifier resolution
- **Tier-to-modifier-pool mapping**: data structure for which modifiers are available at each tier with weights
- **Frame grid sizes**: need experimentation (current nodes range 10x5 to 60x8 to 40x25)
- **Block sizes**: standardized or freeform?
- **Content budget**: how many frames, blocks, skeletons for adequate variety
- **Portal sub-level format**: small arena? linear gauntlet? Mini-frames?
- **Content authoring pipeline**: tooling for designing frames, blocks, skeletons efficiently
- **Integration with existing system**: what of current `NodeLayout` RON stays, what's replaced?

## Dependencies
- Depends on: protocol & hazard system design (must be resolved first)
- Depends on: existing node generation system
- Blocks: Phase 8 (content & variety), Phase 9 (roguelite progression)
- Related: Phase 10 (boss nodes) — boss nodes cap each tier, fully hand-designed

## Terminology Additions
- **Volatile** — node type beyond active; high-danger, unpredictable late-game nodes
- **Tier** — a group of 4 non-boss nodes + 1 boss node; defines difficulty level
- **Portal cell** — required cell leading to a small sub-level (zero nesting)
- **Frame** — outermost node layout template, defines block slot positions
- **Block** — local cell arrangement filling a frame slot, tiered by minimum tier
- **Skeleton** — per-cell-position type constraint within a block

## Status
`[NEEDS DETAIL]` — design intent captured, needs API contracts, data structures, RON formats, and implementation-level detail

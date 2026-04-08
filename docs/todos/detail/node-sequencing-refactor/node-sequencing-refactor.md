# Node Sequencing Refactor

## Summary
Replace full upfront node sequence generation with per-tier batching. Tiers define difficulty progression and node type escalation across a run.

## Context
Currently all nodes for a run are generated upfront. This needs to change to a Balatro-inspired tier system where each tier is generated when the player reaches it, allowing difficulty to scale dynamically and enabling mechanics like tier regression.

## Sub-Documents

### Design
- [Tier Structure & Progression](tiers.md) — tier structure, node type ramp, tier state resources, infinite scaling
- [Cell Types](cell-types.md) — modifier introduction timeline, tier pool weights, constraint resolution
- [Node Generation](node-generation.md) — two-layer composition system (frames/blocks), boss nodes, portal sub-levels

### RON Formats
- [Frame Format](format-frame.md) — `FrameDef` RON spec, examples, constraint enum
- [Block Format](format-block.md) — `BlockDef` RON spec, examples, tier pools, sequence definitions

### Implementation
- [Implementation Waves](waves.md) — 7 waves (0-6), sub-wave breakdown, dependency graph
- [RNG Architecture](rng-architecture.md) — hierarchical seed derivation, per-domain RNG, system-by-system migration

### Research
- [RNG Usage Research](research/rng-usage.md) — current RNG call sites, risks, recommendations

## What's Decided

### Tier System
- Tier structure (4+boss, 8 tiers, infinite continuation)
- Node type ramp (PPPA through VVVV)
- Cell types and introduction timeline (7 types, tiers 1-7)
- Portal cell rules (required, zero nesting, tier N-2 difficulty pool, cap 4)
- Boss nodes are fully hand-designed with code-driven brains

### Composition Architecture
- **Standardized block sizes** — starting set: Small (4x3, 4x4), Medium (6x4, 6x6, 8x4), Large (8x6, 10x5, 10x8). Iterate through playtesting.
- **Tier-ranged blocks** — blocks have both `min_tier` and `max_tier`. Simple blocks don't pollute volatile nodes; complex blocks don't appear in early tiers.
- **Frames are full grids** — mix of hand-placed cells (with behavior constraints) and algorithmic slots. Not just slots with gaps — you can draw fixed cells around open slots (e.g., border cells framing three procedural slots).
- **Frame format** — inline coordinate lists for fixed cells and slots. More verbose than a visual grid but unambiguous, simpler for tooling, no character encoding limits. See [format-frame.md](format-frame.md).
- **Inline cell constraints** — each cell position in a block (or fixed cell in a frame) can carry:
  - `Any` — unconstrained, pick from tier pool
  - `MustInclude([Armored, Volatile])` — resolved cell must include these behaviors
  - `MustNotInclude([Portal])` — resolved cell must not include these behaviors
  - `MustNotInclude(Any)` — plain cell, no modifiers, HP only
  - No `Exact` constraint — constraints guide resolution, not dictate it
- **Axis-aligned slot splitting** — frame slots can be split via weighted random (heavily biased toward fewer splits: Single ~62%, OneSplit ~25%, TwoSplits ~13%). Volatile/high-tier nodes shift weights toward more splits.
- **Modifier combination rules** — deny-list in code. All combos valid by default; only list pairs that conflict or are degenerate. No resource needed.

### Sequence Scoping
- **Block-authored sequences** — blocks can define explicit Sequence(1), Sequence(2) within themselves (block-scoped, IDs local to the block)
- **Generator-assigned sequences** — cells can be marked "eligible for sequence" without explicit ordering. The generator assigns sequence numbers during composition, potentially spanning blocks within a frame.
- Both levels coexist: authored sequences stay local, generator sequences can cross block boundaries.

### Resolution Pipeline
- Constraint + tier modifier pool → `Vec<CellBehavior>` → `Cell::builder().with_behaviors(behaviors)`
- Node generator speaks the cell domain's language (CellBehavior enum)
- No intermediate CellRecipe — direct to builder

### Seed Propagation
- **Hierarchical seed derivation**: run_seed → tier_seed → node_seed → slot_seed → cell_seed
- Each level derives from parent + index, so changing one node's generation doesn't cascade to others
- All node generation for a tier happens in a single deterministic pass
- Full seed derivation tree and per-domain RNG resources in [rng-architecture.md](rng-architecture.md)

### Tier State
- **Split into two resources**: `TierConfig` (tier level, modifier pool, hazard stack — what this tier looks like) and `RunProgress` (current tier, node index within tier, total nodes cleared — where the player is)
- `TierConfig` rebuilt when `RunProgress` crosses a tier boundary

### Portal Sub-Levels
- Portal sub-levels ARE frames — same FrameDef format, tagged `portal: true`
- Use tier N-2 pool of available frames/blocks (so a tier 5 portal plays at tier 3 difficulty)
- Zero nesting enforced: portal frames cannot have Portal constraints in their cells
- Same composition machinery as regular frames, just smaller grid

### Migration
- Clean break from NodeLayout — new system replaces it entirely
- Current hand-designed nodes become frames (hand-placed cells, no slots) or boss layouts
- Boss nodes stay hand-designed (similar to current but new format)
- Delete old NodeLayout code after migration

### Content Budget (starting point — iterate through playtesting)
- **Frames**: ~12 passive, ~12 active, ~24 volatile (volatile gets more — carries infinite scaling)
- **Blocks**: ~229 total, scaling up at higher tiers where infinite runs need more variety:

  | Tier | Small (4x3,4x4) | Medium (6x4,6x6,8x4) | Large (8x6,10x5,10x8) | Total |
  |------|------------------|-----------------------|------------------------|-------|
  | 1    | 2 each           | 3 each                | 4 each                 | 25    |
  | 2    | 2 each           | 3 each                | 4 each                 | 25    |
  | 3    | 2 each           | 3 each                | 4 each                 | 25    |
  | 4    | 3 each           | 4 each                | 5 each                 | 33    |
  | 5    | 3 each           | 4 each                | 6 each                 | 36    |
  | 6    | 4 each           | 5 each                | 6 each                 | 41    |
  | 7    | 4 each           | 5 each                | 7 each                 | 44    |

  - Infinite runs (tier 7+) see 44 block options per size — best variety where players spend the most time
  - Portal sub-levels at tier N-2 see the corresponding tier's block count (e.g., tier 7 portal → tier 5 pool = 36/size)
  - Each block has min_tier AND max_tier — generator filters by current tier
  - Larger sizes get more blocks because single-slot resolution (~62%) favors them
- **Variety multipliers**: constraint resolution (same block, different modifiers per tier), slot splitting, frame × block combinatorics

### Authoring Pipeline
- MVP: RON files + `cargo dev` with hot-reload (frame/block changes update in-game immediately)
- Grid editor tool is a future todo — not part of this work item
- Hot-reload behavior: if a frame in the sequence changes, update it; if on a node whose frame/block changes, kill and respawn contents

### RNG Architecture
Decided — full implementation plan in [rng-architecture.md](rng-architecture.md). Summary: replace flat single-stream `GameRng` with hierarchical seed derivation + per-domain RNG resources (`FxRng`, `NodeSequenceRng`, `NodeGenRng`, `BoltRng`, `ChipRng`) + ephemeral per-effect-fire RNG. 7-phase incremental migration plan (phases 1-5 can land before node sequencing, phase 6 with it).

## Status
`ready` — all architectural decisions resolved, implementation waves planned. See [waves.md](waves.md).

## Dependencies
- Depends on: cell builder pattern (provides Cell::builder() and CellBehavior enum)
- Depends on: protocol & hazard system (must be implemented — Wave 3 wires tier boundary triggers for hazard selection and protocol offerings)
- Blocks: Phase 8 (content & variety), Phase 9 (roguelite progression)
- Related: Phase 10 (boss nodes) — boss nodes cap each tier, fully hand-designed
- Note: Waves 0-2 (RNG migration, data types, composition engine) can start before protocols/hazards are implemented

## Post-Landing: Fill Tier Regression Stub
The Tier Regression protocol is scaffolded with a stub — config resource, activation, offering all work, but the system that actually modifies `NodeSequence` to replay a lower tier is a no-op. After this todo lands and `NodeSequence` supports runtime tier manipulation, fill in the stub at `protocol/protocols/tier_regression.rs`. See `docs/todos/detail/mod-system-design/protocols/tier_regression.md` for the full design.

## Terminology Additions
- **Volatile** — node type beyond active; high-danger, unpredictable late-game nodes
- **Tier** — a group of 4 non-boss nodes + 1 boss node; defines difficulty level
- **Portal cell** — required cell leading to a small sub-level (zero nesting)
- **Frame** — outermost node layout template; full grid with fixed cells + algorithmic block slots
- **Block** — local cell arrangement filling a frame slot; tiered by minimum tier; cell positions carry inline constraints (`Any`, `MustInclude`, `MustNotInclude`)


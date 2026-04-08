# Node Generation

Assumes hazards and protocols already exist as implemented systems.

## Three-Tier Composition System

Nodes are composed from three layers: **Frames > Blocks > Skeletons**.

### Frames (outermost container)
- Define overall node shape and where block slots go
- 10 frames per node type (passive, active, volatile) = 30 total
- **Boss nodes are fully hand-designed** — no frame/block/skeleton system (see Boss Nodes below)
- Max grid size ~40x25 (current BossArena size). All frames designed against a max grid, with `entity_scale` used to "zoom in" on smaller layouts within the same grid space.
- Exact frame sizes TBD through experimentation — current nodes range from 10x5 to 60x8 to 40x25

### Blocks (fit into frame slots)
- Define a local cell arrangement within a frame slot
- **Tiered**: each block has a minimum tier it can appear at (harder/more complex blocks appear later)
- Various sizes — a frame slot area can be filled by one block or subdivided (e.g., a 4x6 area can be filled by one 4x6 block, or 2x 2x6 blocks, or 2x 4x3 blocks, etc.)
- Each cell position within a block has a **skeleton** defining what it can be

### Skeletons (cell type constraints per position)
- The smallest unit — a rule for what a single cell position can be
- Examples:
  - `[Lock]` — must be a lock cell
  - `[Portal]` — must be a portal cell
  - `[Standard, Armored, Volatile]` — pick one based on tier weights
  - `[Any]` — any cell type from the tier's available pool
  - `[Empty]` — guaranteed open space for bolt movement

### Composition Flow
1. Pick a **frame** for the node type (seeded)
2. For each frame slot, pick a **block** (or subdivide and pick multiple blocks) whose min-tier fits and whose dimensions match (seeded)
3. For each cell position in the block, resolve the **skeleton** constraint against the tier's available cell pool
4. Apply tier modifiers (HP scaling, etc.)

### Content Budget
- ~30 frames (10 per non-boss node type)
- Blocks and skeletons TBD
- Combinatorial variety from composition + cell type resolution per tier

## Boss Nodes
Boss nodes are **fully hand-designed** — custom layouts with a code-driven "brain" that controls behavior. Bosses are not composed from frames/blocks/skeletons.

Examples of boss behavior:
- Space Invaders-style: entire cell grid moves as a unit
- Pattern-based attacks with phase transitions
- Layout that changes mid-fight (cells rearrange, new cells spawn)

Each boss is a unique bespoke experience per tier. Boss design is scoped under Phase 10 (boss nodes & advanced mechanics).

## Needs Detail

### RON Formats
- **Frame RON format** — how are block slots defined? Tagged rectangles with position, size, and constraints? What does a frame file actually look like?
- **Block RON format** — how is a grid of skeletons authored? Is each cell position a skeleton inline, or do skeletons reference named patterns? What metadata (min tier, tags)?
- **Skeleton format** — is `[Standard, Armored, Volatile]` a RON enum? A list of strings? How do weights per tier work? How does skeleton format interact with the cell modifier RON format (from cell builder todo)?

### Composition Algorithm
- **Block subdivision** — when a 4x6 frame slot can be filled by 2x 2x6 blocks, how is that expressed? Does the frame define allowed splits, or does the generator figure it out?
- **Seed propagation** — how does the run seed flow through frame selection → block selection → skeleton → modifier resolution? Must be deterministic for seed sharing.
- **Frame slot constraints** — can a frame slot require specific block properties (e.g., "must contain a portal")?
- **Tier-to-pool mapping** — data structure for which modifiers are available at each tier and their weights for skeleton resolution

### Sizing & Content
- **Exact frame grid sizes** — need experimentation to determine what sizes work at each node type (current nodes range 10x5 to 60x8 to 40x25)
- **Block sizes** — standardized dimensions? Or freeform with frame slots defining the size?
- **Content authoring pipeline** — tooling for designing frames, blocks, skeletons efficiently
- **Content budget** — how many frames, blocks, skeletons needed for adequate variety?

### Integration
- **Integration with existing system** — what of the current `NodeLayout` RON format stays? What's replaced?
- **Cell builder integration** — how does skeleton resolution feed into `Cell::builder()`? Does the generator call `.volatile().armored(2)` directly, or pass a list of `CellBehavior` enums?

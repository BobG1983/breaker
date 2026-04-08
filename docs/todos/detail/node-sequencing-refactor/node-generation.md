# Node Generation

Assumes hazards and protocols already exist as implemented systems.

## Two-Layer Composition System

Nodes are composed from two layers: **Frames > Blocks**.

### Frames (outermost container)
- Define a full grid where some positions are **hand-placed cells** and others are **block slots**
- Hand-placed cells can have behavior constraints (same `Any`/`MustInclude`/`MustNotInclude` as block cells)
- Enables designs like: border cells framing three procedural slots, or a corridor of fixed cells with pockets of procedural content
- **Content budget**: ~12 passive, ~12 active, ~24 volatile (volatile gets more — carries infinite scaling)
- **Boss nodes are fully hand-designed** — no frame/block system (see Boss Nodes below)
- Max grid size ~40x25 (current BossArena size). All frames designed against a max grid, with `entity_scale` used to "zoom in" on smaller layouts within the same grid space.
- Starting frame grids (iterate through playtesting): Passive ~16x10/20x12, Active ~24x14/30x16, Volatile ~32x18/40x20

#### Frame Format

Inline coordinate lists for fixed cells and slots. See [format-frame.md](format-frame.md) for the full `FrameDef` RON spec, constraint enum, and examples.

```ron
FrameDef(
  node_type: Passive,
  grid: (12, 6),
  fixed: [
    ((0, 0), Any), ((1, 0), Any), // ... border cells
    ((0, 3), MustInclude([Armored])),
  ],
  slots: [
    Slot(id: "left",  origin: (2, 1), size: S4x4),
    Slot(id: "right", origin: (7, 1), size: S4x4),
  ],
  portal: false,
)
```

### Blocks (fit into frame slots)
- Define a local cell arrangement within a frame slot
- **Standardized sizes**: Small (4x3, 4x4), Medium (6x4, 6x6, 8x4), Large (8x6, 10x5, 10x8). Iterate through playtesting.
- **Tier-ranged**: each block has both a `min_tier` and `max_tier`. Simple blocks don't appear in volatile nodes; complex blocks don't appear in early tiers. Generator filters by current tier fitting within the block's range.
- ~229 blocks total (starting budget): scaling up at higher tiers (~25/tier early, ~44/tier late), weighted toward larger sizes. See main doc content budget table.
- Each cell position carries an inline constraint. See [format-block.md](format-block.md) for the full `BlockDef` RON spec, constraint enum, tier pool format, and examples.

```ron
BlockDef(
  size: S6x4,
  min_tier: 3,
  max_tier: 6,
  cells: [
    ((0, 0), Any),
    ((1, 0), MustInclude([Armored])),
    ((2, 0), MustInclude([Armored])),
    ((3, 0), Any),
    ((0, 1), MustNotInclude([Portal])),
    ((1, 1), Any),
    // positions not listed = empty space
  ],
  sequences: [],
)
```

### Slot Splitting
Frame slots can be split into sub-slots of standard block sizes via **axis-aligned splitting**:
- Vertical split: 8x6 → 2x 4x6
- Horizontal split: 8x6 → 2x 8x3
- Recursive: 8x3 → 2x 4x3

**Algorithm**: weighted random from valid splits, **heavily biased toward fewer splits**:
- `Single` (no split): weight ~5
- `OneSplit` (2 sub-blocks): weight ~2
- `TwoSplits` (4 sub-blocks): weight ~1
- Result: ~62% of slots stay whole, splitting is the exception for variety
- Volatile/high-tier nodes shift weights slightly toward more splitting
- Split decision seeded per-slot (`hash(node_seed, slot_index, "split")`)

**Content budget implication**: front-load larger block sizes (more authored, less repetition when they're picked ~62% of the time). Smaller blocks get reused more when splitting does happen, so fewer are needed.

**Note**: an alternative is to skip splitting entirely and have frame authors use multiple adjacent slots. This is simpler but puts more design burden on frames. Current decision: support splitting but bias toward single blocks. Revisit if content authoring is painful.

### Sequence Scoping
- **Block-authored sequences**: blocks can define explicit Sequence(1), Sequence(2) (IDs local to the block)
- **Generator-assigned sequences**: cells marked "eligible for sequence" get ordering assigned during composition, potentially spanning blocks within a frame
- Both levels coexist

### Composition Flow
1. Pick a **frame** for the node type (seeded: `hash(node_seed, "frame")`)
2. For each frame slot, optionally split into sub-slots of standard block sizes
3. For each sub-slot, pick a **block** whose min-tier fits and whose size matches (seeded: `hash(node_seed, slot_index)`)
4. For each cell position (both frame fixed cells and block cells), resolve constraints against the tier's modifier pool → `Vec<CellBehavior>` (seeded: `hash(slot_seed, cell_pos)`)
5. Call `Cell::builder().with_behaviors(behaviors)` for each cell
6. Apply tier modifiers (HP scaling, etc.)
7. Assign generator-level sequence ordering across eligible cells

### Seed Propagation
Hierarchical derivation ensures stability across content changes:
```
run_seed: u64
tier_seed  = hash(run_seed, tier_index)
node_seed  = hash(tier_seed, node_index)
slot_seed  = hash(node_seed, slot_index)
cell_seed  = hash(slot_seed, cell_pos)
```
Changing one node's generation doesn't cascade to others. All generation for a tier happens in a single deterministic system pass.

## Portal Sub-Levels
Portal sub-levels ARE frames — same FrameDef format, tagged `portal: true`. They use the tier **N-2** pool of available frames/blocks (so a tier 5 portal plays at tier 3 difficulty). Zero nesting: portal frames cannot have Portal constraints in their cells.

## Boss Nodes
Boss nodes are **fully hand-designed** — custom layouts with a code-driven "brain" that controls behavior. Bosses are not composed from frames/blocks.

Examples of boss behavior:
- Space Invaders-style: entire cell grid moves as a unit
- Pattern-based attacks with phase transitions
- Layout that changes mid-fight (cells rearrange, new cells spawn)

Each boss is a unique bespoke experience per tier. Boss design is scoped under Phase 10 (boss nodes & advanced mechanics).

## Migration from NodeLayout
Clean break — new system replaces NodeLayout entirely. Current hand-designed nodes become frames (all fixed cells, no slots) or boss layouts. Delete old NodeLayout code after migration.

## Authoring Pipeline
- MVP: RON files authored by hand + `cargo dev` with hot-reload
- Hot-reload: if a frame/block RON changes while in-game, kill and respawn the affected node's contents
- Grid editor tool is a future todo (not part of this work item)

## Resolved
- **RNG architecture**: fully designed — see [rng-architecture.md](rng-architecture.md). Hierarchical seed derivation with per-domain RNG resources. `NodeGenRng` handles frame/block/cell resolution seeded from `hash(run_seed, tier_index, node_index)`.

# Block RON Format

A block defines a local cell arrangement that fills a frame slot. Each cell position carries a constraint that guides modifier resolution from the tier pool.

## File Convention

```
assets/blocks/<size>/<name>.block.ron
```

Examples:
- `assets/blocks/s6x4/wall_armored.block.ron`
- `assets/blocks/s8x6/arena_mixed_t5.block.ron`
- `assets/blocks/s4x3/simple_scatter.block.ron`

Organized by size so the generator can efficiently enumerate blocks matching a slot size.

## Format

```ron
BlockDef(
  // Standard block size. Must match the directory name.
  size: S6x4,

  // Tier range this block can appear in.
  // min_tier: earliest tier this block is eligible (inclusive).
  // max_tier: latest tier this block is eligible (inclusive).
  // Generator filters blocks by: min_tier <= current_tier <= max_tier.
  min_tier: 3,
  max_tier: 6,

  // Cell positions with constraints.
  // Each entry is a (col, row) position relative to the block's origin + a CellConstraint.
  // Positions not listed default to Any (unconstrained, resolved from tier pool).
  // Positions within the block grid that have no entry and are not listed = empty space (no cell).
  cells: [
    // Row 0: armored wall
    ((0, 0), Any),
    ((1, 0), MustInclude([Armored])),
    ((2, 0), MustInclude([Armored])),
    ((3, 0), MustInclude([Armored])),
    ((4, 0), MustInclude([Armored])),
    ((5, 0), Any),

    // Row 1: open with volatile anchors
    ((0, 1), MustInclude([Volatile])),
    ((5, 1), MustInclude([Volatile])),

    // Row 2: scattered cells
    ((1, 2), Any),
    ((2, 2), Any),
    ((3, 2), Any),
    ((4, 2), Any),

    // Row 3: full row
    ((0, 3), Any),
    ((1, 3), Any),
    ((2, 3), Any),
    ((3, 3), Any),
    ((4, 3), Any),
    ((5, 3), Any),
  ],

  // Block-scoped sequence definitions (optional).
  // These are explicit sequences authored by the block designer.
  // Sequence IDs are local to this block.
  // Cells listed here MUST also appear in the cells list above.
  sequences: [
    Sequence(
      id: 1,
      cells: [(1, 0), (2, 0), (3, 0), (4, 0)],
    ),
  ],
)
```

## BlockSize Enum

```ron
// Standard block sizes (cell grid units, not pixels).
// Pixel size determined by entity_scale at the frame level.
S4x3   // 4 cols × 3 rows
S4x4   // 4 cols × 4 rows
S6x4   // 6 cols × 4 rows
S6x6   // 6 cols × 6 rows
S8x4   // 8 cols × 4 rows
S8x6   // 8 cols × 6 rows
S10x5  // 10 cols × 5 rows
S10x8  // 10 cols × 8 rows
```

## CellConstraint Enum

Same as in frames — see [format-frame.md](format-frame.md#cellconstraint-enum).

```ron
Any                                  // unconstrained, resolve from tier pool
MustInclude([Armored, Volatile])     // must have these, may get more from pool
MustNotInclude([Portal, Magnetic])   // must NOT have these, rest from pool
MustNotInclude(Any)                  // plain cell — no modifiers, HP only
```

## Cell Position Rules

1. **Grid bounds**: all cell positions must fit within `(0, 0)` to `(size.cols - 1, size.rows - 1)`.
2. **Explicit positions only**: a position in the cells list = a cell exists there. A position NOT in the list = empty space (no cell). There is no implicit "fill all positions."
3. **No duplicates**: each `(col, row)` can appear at most once.
4. **Density**: blocks should have enough cells to be interesting but enough empty space for bolt movement. No hard rule — playtesting determines what feels right.

## Sequence Rules

1. **Block-scoped**: sequence IDs are local to the block. Two blocks in the same frame can both have `Sequence(id: 1)` — they're independent groups.
2. **Cells must exist**: every position in a sequence's `cells` list must also appear in the block's `cells` list.
3. **Ordering**: the order of positions in the `cells` list defines the sequence order. First position = Sequence step 1, etc.
4. **Optional**: blocks don't need sequences. Most blocks won't have them.
5. **Generator-assigned sequences**: cells can also be marked as "eligible for sequence" via a constraint. The generator assigns cross-block sequence ordering during frame composition — this is separate from block-authored sequences.

## Tier Range Guidelines

| Tier Range | Block Character |
|------------|-----------------|
| 1-3 | Simple layouts, few constraints, mostly `Any`. Learning-friendly. |
| 2-5 | Moderate complexity. Some `MustInclude` constraints. First sequences. |
| 4-7 | Dense, constrained. Multiple modifier requirements. Strategic empty space. |
| 5+ (no max) | Complex, high-density. Heavy constraints. Made for volatile nodes + infinite runs. |

Use `max_tier` to prevent simple blocks from appearing in high tiers where they'd feel out of place. Blocks designed for infinite runs should have no `max_tier` (or a very high value).

## Minimal Example

A simple tier 1-3 block with all unconstrained cells:

```ron
BlockDef(
  size: S4x3,
  min_tier: 1,
  max_tier: 3,
  cells: [
    ((0, 0), Any), ((1, 0), Any), ((2, 0), Any), ((3, 0), Any),
    ((0, 1), Any),                                ((3, 1), Any),
    ((0, 2), Any), ((1, 2), Any), ((2, 2), Any), ((3, 2), Any),
  ],
  sequences: [],
)
```

## Complex Example

A tier 5+ block with armored walls, volatile corners, a sequence, and strategic gaps:

```ron
BlockDef(
  size: S8x6,
  min_tier: 5,
  max_tier: 8,
  cells: [
    // Row 0: volatile corners, armored bridge
    ((0, 0), MustInclude([Volatile])),
    ((1, 0), MustInclude([Armored])),
    ((2, 0), MustInclude([Armored])),
    ((3, 0), MustInclude([Armored])),
    ((4, 0), MustInclude([Armored])),
    ((5, 0), MustInclude([Armored])),
    ((6, 0), MustInclude([Armored])),
    ((7, 0), MustInclude([Volatile])),

    // Row 1: sequence targets with gaps
    ((1, 1), Any),
    ((3, 1), Any),
    ((5, 1), Any),
    ((7, 1), Any),

    // Row 2: open (bolt movement corridor)

    // Row 3: scattered cells, no portals allowed
    ((0, 3), MustNotInclude([Portal])),
    ((2, 3), Any),
    ((4, 3), Any),
    ((6, 3), MustNotInclude([Portal])),

    // Row 4: sequence chain
    ((1, 4), Any),
    ((3, 4), Any),
    ((5, 4), Any),

    // Row 5: volatile corners, armored bridge (mirror of row 0)
    ((0, 5), MustInclude([Volatile])),
    ((1, 5), MustInclude([Armored])),
    ((2, 5), MustInclude([Armored])),
    ((3, 5), MustInclude([Armored])),
    ((4, 5), MustInclude([Armored])),
    ((5, 5), MustInclude([Armored])),
    ((6, 5), MustInclude([Armored])),
    ((7, 5), MustInclude([Volatile])),
  ],
  sequences: [
    Sequence(
      id: 1,
      cells: [(1, 4), (3, 4), (5, 4)],
    ),
  ],
)
```

## Tier Pool RON Format

The tier modifier pool defines which modifiers are available at each tier and their weights for constraint resolution.

```
assets/tier_pools.ron
```

```ron
TierPools([
  TierPool(
    tier: 1,
    modifiers: [
      (Volatile, 0.3),
      // 70% chance of no modifier (plain cell)
    ],
  ),
  TierPool(
    tier: 2,
    modifiers: [
      (Volatile, 0.25),
    ],
  ),
  TierPool(
    tier: 3,
    modifiers: [
      (Volatile, 0.2),
      (Sequence, 0.15),
    ],
  ),
  TierPool(
    tier: 4,
    modifiers: [
      (Volatile, 0.2),
      (Sequence, 0.12),
      (Survival, 0.12),
    ],
  ),
  TierPool(
    tier: 5,
    modifiers: [
      (Volatile, 0.15),
      (Sequence, 0.12),
      (Survival, 0.12),
      (Armored,  0.15),
    ],
  ),
  TierPool(
    tier: 6,
    modifiers: [
      (Volatile, 0.15),
      (Sequence, 0.10),
      (Survival, 0.10),
      (Armored,  0.12),
      (Phantom,  0.10),
    ],
  ),
  TierPool(
    tier: 7,
    modifiers: [
      (Volatile, 0.12),
      (Sequence, 0.10),
      (Survival, 0.10),
      (Armored,  0.12),
      (Phantom,  0.08),
      (Magnetic, 0.08),
    ],
  ),
  // Tier 8+ uses tier 7 pool with escalating weights (handled in code)
])
```

The remaining probability (1.0 minus sum of weights) = chance of a plain cell with no modifiers. Higher tiers have more total modifier weight = fewer plain cells = harder nodes.

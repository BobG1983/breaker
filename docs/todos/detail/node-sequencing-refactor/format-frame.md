# Frame RON Format

A frame defines the outermost node layout: a grid with fixed cell positions and block slots. The generator fills slots with blocks; fixed cells are resolved directly against the tier pool.

## File Convention

```
assets/frames/<node_type>/<name>.frame.ron
```

Examples:
- `assets/frames/passive/corridor.frame.ron`
- `assets/frames/volatile/chaos_arena.frame.ron`
- `assets/frames/portal/gauntlet_01.frame.ron`

## Format

```ron
FrameDef(
  // Which node types can use this frame.
  // Generator picks from frames matching the current node type.
  node_type: Passive,

  // Grid dimensions in cell units.
  // All positions (fixed cells + slots) must fit within this grid.
  grid: (20, 12),

  // Fixed cell positions — cells placed by the frame author.
  // Each entry is a (col, row) position + a CellConstraint.
  // Constraint determines what modifiers the cell gets from the tier pool.
  // Positions not listed here and not covered by a slot are empty space.
  fixed: [
    // Border cells — unconstrained, resolved from tier pool
    ((0, 0), Any),
    ((1, 0), Any),
    ((2, 0), Any),
    ((3, 0), Any),
    // ...

    // Constrained cells — frame author wants specific modifiers here
    ((0, 3), MustInclude([Armored])),
    ((0, 4), MustInclude([Armored])),

    // Cell that must NOT be a portal (e.g., near the border)
    ((5, 0), MustNotInclude([Portal])),
  ],

  // Block slots — rectangular areas filled by the generator with blocks.
  // Each slot has a standard BlockSize. Generator picks a block (or splits
  // the slot into smaller blocks) whose tier range includes the current tier.
  slots: [
    Slot(
      id: "center",
      origin: (4, 2),
      size: S8x6,
    ),
    Slot(
      id: "right_pocket",
      origin: (14, 3),
      size: S6x4,
    ),
  ],

  // Portal frame flag (default: false).
  // Portal frames are used for portal sub-levels.
  // Portal frames MUST NOT have MustInclude([Portal]) on any fixed cell.
  // Portal frames use the tier N-2 pool for block/modifier resolution.
  portal: false,
)
```

## CellConstraint Enum

```ron
// Unconstrained — resolve entirely from tier modifier pool.
// May get zero modifiers (plain cell) or any combination the pool rolls.
Any

// Cell must include ALL listed behaviors.
// Additional behaviors may be added from the tier pool.
MustInclude([Armored, Volatile])

// Cell must NOT include ANY of the listed behaviors.
// Other behaviors resolved normally from tier pool.
MustNotInclude([Portal, Magnetic])

// Cell must have NO modifiers at all — plain cell with HP only.
// Shorthand for "exclude every modifier."
MustNotInclude(Any)
```

## Rules

1. **Grid bounds**: all fixed cell positions and slot rectangles must fit within `(0,0)` to `(grid.0 - 1, grid.1 - 1)`.
2. **No overlap**: fixed cells and slots must not overlap. A position is either a fixed cell, part of a slot, or empty space.
3. **Slot sizes**: must be a valid `BlockSize` enum variant.
4. **Portal frames**: `portal: true` frames cannot have `MustInclude` constraints that include `Portal`. The zero-nesting rule is enforced at load time.
5. **Node type**: each frame declares which node type it's for. A frame with `node_type: Passive` is only used for passive node slots in the tier.

## Minimal Example

A small passive frame with one slot and a few border cells:

```ron
FrameDef(
  node_type: Passive,
  grid: (8, 6),
  fixed: [
    // top border
    ((0, 0), Any), ((1, 0), Any), ((2, 0), Any), ((3, 0), Any),
    ((4, 0), Any), ((5, 0), Any), ((6, 0), Any), ((7, 0), Any),
    // bottom border
    ((0, 5), Any), ((1, 5), Any), ((2, 5), Any), ((3, 5), Any),
    ((4, 5), Any), ((5, 5), Any), ((6, 5), Any), ((7, 5), Any),
  ],
  slots: [
    Slot(
      id: "main",
      origin: (0, 1),
      size: S8x4,
    ),
  ],
  portal: false,
)
```

## Complex Example

A volatile frame with mixed fixed cells, multiple slots, and constraints:

```ron
FrameDef(
  node_type: Volatile,
  grid: (24, 14),
  fixed: [
    // Armored corridor walls on left
    ((0, 3), MustInclude([Armored])),
    ((0, 4), MustInclude([Armored])),
    ((0, 5), MustInclude([Armored])),
    ((0, 6), MustInclude([Armored])),
    ((0, 7), MustInclude([Armored])),
    ((0, 8), MustInclude([Armored])),
    ((0, 9), MustInclude([Armored])),
    ((0, 10), MustInclude([Armored])),

    // Scattered unconstrained cells in open area
    ((12, 0), Any),
    ((15, 0), Any),
    ((18, 0), Any),
    ((12, 13), Any),
    ((15, 13), Any),
    ((18, 13), Any),
  ],
  slots: [
    // Large central arena
    Slot(
      id: "arena",
      origin: (2, 2),
      size: S10x8,
    ),
    // Side pockets
    Slot(
      id: "left_pocket",
      origin: (14, 1),
      size: S4x4,
    ),
    Slot(
      id: "right_pocket",
      origin: (14, 8),
      size: S4x4,
    ),
    // Top strip
    Slot(
      id: "top",
      origin: (20, 3),
      size: S4x3,
    ),
  ],
  portal: false,
)
```

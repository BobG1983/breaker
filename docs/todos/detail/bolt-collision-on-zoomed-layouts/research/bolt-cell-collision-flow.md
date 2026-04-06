## Behavior Trace: Bolt-Cell Collision Detection

Bevy version: **0.18.1** (from `breaker-game/Cargo.toml`)

---

### Trigger

`FixedUpdate` fires. The `bolt_cell_collision` system runs after
`PhysicsSystems::MaintainQuadtree`, after `BreakerSystems::Move`, and after
`PhysicsSystems::EnforceDistanceConstraints`. It processes every active bolt
in one shot — no message triggers it.

---

### System Chain

1. **`maintain_quadtree`** (`RantzPhysics2dPlugin`, `FixedUpdate`,
   `PhysicsSystems::MaintainQuadtree`)
   Reads: `Added<Aabb2D>`, `Changed<GlobalPosition2D>`, `Changed<CollisionLayers>`,
   `RemovedComponents<Aabb2D>`
   Writes: `CollisionQuadtree` (insert / remove / update)
   Effect: keeps the quadtree in sync with the world. Cells are inserted here
   using `GlobalPosition2D` as the center and `Aabb2D.half_extents` as the size.

2. **`bolt_cell_collision`** (`BoltPlugin`, `FixedUpdate`,
   `BoltSystems::CellCollision`)
   Reads: `CollisionQuadtree`, `BoltCollisionData` (position, velocity,
   `BoltRadius`, `NodeScalingFactor`, `ActiveDamageBoosts`, `PiercingRemaining`,
   `LastImpact`, `BoltBaseDamage`), `Time<Fixed>`
   Writes: `Position2D` and `Velocity2D` on bolt, `PiercingRemaining` on bolt,
   `LastImpact` (inserted via `Commands` if absent, mutated if present),
   `BoltImpactCell` message, `DamageCell` message
   Effect: moves bolt, reflects or pierces.

3. **Downstream message consumers** (next frame, various systems)
   `BoltImpactCell` → effect dispatch (chain lightning, mirror protocol, etc.)
   `DamageCell` → `handle_cell_hit` (cells domain) → decrements `CellHealth`,
   destroys cell when HP reaches zero.

---

### Data Flow

```
NodeLayout.entity_scale
    └─ apply_node_scale_to_bolt (OnEnter NodeState::Loading)
           └─ NodeScalingFactor(f32) component on Bolt entity

BoltDefinition.radius
    └─ Bolt::builder() → BaseRadius(f32) + Aabb2D(half=radius, radius)
           └─ sync_bolt_scale (Update) → Scale2D(visual only)

                          ┌─────────────────────────────────────────┐
FixedUpdate frame:        │                                         │
                          │  maintain_quadtree                      │
  Cell spawn              │  reads: GlobalPosition2D, Aabb2D,       │
  (OnEnter Playing)       │         CollisionLayers                 │
  ↓                       │  inserts into CollisionQuadtree:        │
  Aabb2D(half_extents =   │    center = GlobalPosition2D            │
    cell_width/2,         │    half_extents = Aabb2D.half_extents   │
    cell_height/2)        │                                         │
                          └─────────────────────────────────────────┘
                                              │
                          ┌─────────────────────────────────────────┐
                          │  bolt_cell_collision                    │
                          │                                         │
                          │  r = BoltRadius * NodeScalingFactor     │
                          │  (falls back to BoltRadius if no scale) │
                          │                                         │
                          │  quadtree.cast_circle(                  │
                          │    position, direction, remaining,      │
                          │    r, collision_layers)                 │
                          │                                         │
                          │  broad-phase: swept AABB (origin ±r     │
                          │    to end ±r) layer-filtered query      │
                          │  narrow-phase: expand each stored AABB  │
                          │    by r (Minkowski), ray-vs-AABB test   │
                          │                                         │
                          │  hit → reflect/pierce                   │
                          │       → BoltImpactCell message          │
                          │       → DamageCell message              │
                          └─────────────────────────────────────────┘
```

---

### Question 1: Is collision point-based or radius-aware?

**Radius-aware (swept-circle CCD).** The system uses
`Quadtree::cast_circle`, which implements Minkowski expansion:

1. **Broad-phase** — computes a swept AABB from `origin ± r` to `end ± r`,
   performs a layer-filtered AABB query to get candidates.
2. **Narrow-phase** — for each candidate, calls `stored_aabb.expand_by(r)`,
   then casts a point ray (`ray_vs_aabb`) against the expanded AABB.

The expansion is equivalent to asking "where does the center of a circle of
radius `r` first touch the cell's AABB?" — i.e., proper circle-vs-AABB CCD.

The `r` value used in `cast_circle` is:

```rust
let bolt_scale = bolt.collision.node_scale.map_or(1.0, |s| s.0);
let r = bolt.collision.radius.0 * bolt_scale;
```

So the collision radius is `BoltRadius * NodeScalingFactor` (or just
`BoltRadius` if no `NodeScalingFactor` component is present).

---

### Question 2: Which components are used?

**Bolt side (read by `bolt_cell_collision`):**

| Component | Type | Role |
|---|---|---|
| `Position2D` (via `SpatialData`) | `rantzsoft_spatial2d` | Bolt center in world space |
| `Velocity2D` (via `SpatialData`) | `rantzsoft_spatial2d` | Bolt direction + speed |
| `BoltRadius` (alias `BaseRadius`) | `shared::size` | Unscaled bolt radius |
| `NodeScalingFactor` | `shared::components` | Optional per-node scale (0.5–1.0) |
| `CollisionLayers` | `rantzsoft_physics2d` | Layer mask (queries CELL_LAYER) |

Note: `Aabb2D` is present on the bolt but is **not read** by
`bolt_cell_collision`. The bolt's `Aabb2D` is used by `maintain_quadtree` to
register the bolt in the quadtree for other systems (e.g.,
`breaker_cell_collision`). The bolt-cell CCD system only reads `BoltRadius`
and `NodeScalingFactor` directly to construct `r`.

**Cell side (registered into `CollisionQuadtree` by `maintain_quadtree`):**

| Component | Type | Role |
|---|---|---|
| `Aabb2D` | `rantzsoft_physics2d` | Cell collision shape (half_extents = cell size / 2) |
| `GlobalPosition2D` | `rantzsoft_spatial2d` | World-space center used as quadtree key |
| `CollisionLayers` | `rantzsoft_physics2d` | Membership mask (CELL_LAYER) |

`CellWidth` and `CellHeight` are **not read** by the collision system at all.
`Aabb2D.half_extents` is the sole source of truth for cell collision geometry.

---

### Question 3: Broadphase and narrowphase detail

**Broadphase** (`Quadtree::cast_circle`, inside `quadtree/tree.rs`):

```rust
let swept_aabb = Aabb2D::from_min_max(
    origin.min(end) - Vec2::splat(radius),
    origin.max(end) + Vec2::splat(radius),
);
let candidates = self.query_aabb_filtered(&swept_aabb, layers);
```

Returns all entities whose stored `Aabb2D` overlaps the swept region **and**
whose `CollisionLayers` membership matches the query mask. The query mask used
by `bolt_cell_collision` is `CollisionLayers::new(0, CELL_LAYER)`.

**Narrowphase** (still inside `cast_circle`):

```rust
let expanded = stored_aabb.expand_by(radius);
if let Some(ray_hit) = expanded.ray_intersect(origin, direction, max_dist) {
    let safe_dist = (ray_hit.distance - CCD_EPSILON).max(0.0);
    let position = origin + direction * safe_dist;
    let remaining = (max_dist - ray_hit.distance).max(0.0);
    // ...
}
```

The stored AABB is the **world-space** AABB: center = `GlobalPosition2D`,
half_extents = `Aabb2D.half_extents` as spawned. Expanding by `r` is the
Minkowski sum of the cell AABB and a circle of radius `r`. A point ray through
this expanded AABB is exactly equivalent to a circle-vs-AABB sweep test.

`CCD_EPSILON = 0.01` is applied as a safety gap to prevent floating-point
re-entry.

**Layer filtering** (`CollisionLayers::interacts_with`):

```
query.mask & entity.membership != 0
```

Cells are spawned with `CollisionLayers::new(CELL_LAYER, BOLT_LAYER)`.
The bolt-cell query uses `CollisionLayers::new(0, CELL_LAYER)`.
`0 (bolt mask) & CELL_LAYER (cell membership) != 0` — cells are returned.
Walls and the breaker have different memberships and are filtered out.

---

### Question 4: Does collision know about `node_scale`?

**For the bolt radius: yes, explicitly.**

`bolt_cell_collision` reads `NodeScalingFactor` from the bolt entity and
multiplies it into `r` before calling `cast_circle`. A bolt with
`BoltRadius(8.0)` and `NodeScalingFactor(0.5)` uses `r = 4.0` for all CCD
intersection tests.

**For cell Aabb2D half_extents: scale is baked in at spawn time.**

Cells do not have `NodeScalingFactor` components. Instead, `compute_grid_scale`
in `spawn_cells_from_layout/system.rs` computes a uniform `scale` factor to fit
the grid into the playfield cell zone. The `cell_width` and `cell_height` passed
to `Aabb2D::new(Vec2::ZERO, Vec2::new(cell_width / 2.0, cell_height / 2.0))` are
already multiplied by this scale. So cell AABB half_extents reflect the actual
rendered size.

**The two scale systems are independent:**

- `NodeLayout.entity_scale` — scales the **bolt and breaker** via
  `NodeScalingFactor`. Applied by `apply_node_scale_to_bolt` on
  `OnEnter(NodeState::Loading)`. Cells are NOT affected by this value.
- `compute_grid_scale` — scales **cell dimensions** to fit the grid in the
  playfield. Baked into `Aabb2D.half_extents` at spawn. Bolt is NOT scaled by
  this value.

These two scales are computed separately and are numerically independent.
A zoomed layout (many rows/columns) may have a small `compute_grid_scale`
factor making cells physically smaller, while `entity_scale` could be 1.0
(full bolt size) or some other value.

---

### System Chain Summary

```
OnEnter(NodeState::Loading):
  spawn_cells_from_layout      → Cell entities with Aabb2D(half = cell_w/2, cell_h/2)
                                  scale baked in via compute_grid_scale
  apply_node_scale_to_bolt     → NodeScalingFactor(entity_scale) on Bolt entities

FixedUpdate (NodeState::Playing):
  maintain_quadtree            → Inserts Cell Aabb2D + GlobalPosition2D into CollisionQuadtree
  bolt_cell_collision          → r = BoltRadius * NodeScalingFactor
                                  cast_circle(pos, dir, dist, r, CELL_LAYER)
                                    broadphase: swept AABB ±r, layer-filtered
                                    narrowphase: expand cell AABB by r, ray test
                                  hit → reflect or pierce
                                  emits BoltImpactCell, DamageCell messages
```

---

### Edge Cases

1. **No `NodeScalingFactor` on bolt** — `map_or(1.0, ...)` means the raw
   `BoltRadius` is used unchanged. Backward-compatible.

2. **Bolt origin inside expanded cell AABB** — `ray_vs_aabb` returns `None`
   when `tmin <= 0.0` (origin inside). This means a bolt that starts inside a
   cell's expanded AABB will not detect a collision that frame. This is the
   "already overlapping" case.

3. **`MAX_BOUNCES = 4` cap** — if geometry creates degenerate bounce loops the
   bolt simply stops iterating. The remaining travel distance is not consumed.

4. **Piercing logic depends on `cell_health` being present** — if a cell has
   no `CellHealth` component, `cell_hp` is `None` and `would_destroy` is
   `false`, so the bolt always reflects (never pierces) against that cell.

5. **`CellWidth` / `CellHeight` components are NOT read by collision** —
   they are only used by debug/hot-reload systems and visual rendering. The
   collision shape is entirely determined by `Aabb2D.half_extents`.

6. **The bolt's own `Aabb2D`** — spawned with `half_extents = (radius, radius)`
   and updated by `maintain_quadtree`. This registers the bolt in the quadtree
   for other collision systems (breaker_cell_collision) that query for bolts by
   spatial region. `bolt_cell_collision` itself does not read the bolt's own
   `Aabb2D` — it reads `BoltRadius` and `NodeScalingFactor` directly.

7. **`sync_bolt_scale` runs in `Update`, not `FixedUpdate`** — it updates
   `Scale2D` (visual rendering size). It does NOT update `Aabb2D.half_extents`
   on the bolt. Therefore the bolt's quadtree entry (used by other systems) may
   lag behind the visual size when `NodeScalingFactor` changes mid-node (which
   does not happen in practice — it is only set at node entry).

---

### Potential Gap: Two Unrelated Scale Systems

`NodeLayout.entity_scale` scales the bolt and breaker only. The grid scale
(`compute_grid_scale`) scales cells only. There is no code path that combines
or cross-references these two values.

If a layout has both `entity_scale < 1.0` (shrinking the bolt) and a grid that
triggers `compute_grid_scale < 1.0` (shrinking cells), these are applied
independently. The collision detection will correctly handle each (bolt radius
is scaled by `entity_scale`, cell AABB is scaled by the grid scale) because
both values feed into `r` and stored AABB half_extents respectively.

However, the two scales are not coordinated. A layout designer setting
`entity_scale` has no direct visibility into what `compute_grid_scale` will
produce for their grid size — and vice versa.

---

### Key Files

- `/Users/bgardner/dev/brickbreaker/breaker-game/src/bolt/systems/bolt_cell_collision/system.rs` —
  main collision loop; reads `BoltRadius * NodeScalingFactor` to compute `r`,
  calls `quadtree.cast_circle`

- `/Users/bgardner/dev/brickbreaker/rantzsoft_physics2d/src/quadtree/tree.rs` —
  `cast_circle` implementation; broadphase swept AABB + narrowphase Minkowski
  expansion (`expand_by(radius)`) + `ray_vs_aabb`

- `/Users/bgardner/dev/brickbreaker/rantzsoft_physics2d/src/ccd/system.rs` —
  `ray_vs_aabb` slab test; `CCD_EPSILON = 0.01`; `reflect` helper

- `/Users/bgardner/dev/brickbreaker/rantzsoft_physics2d/src/systems/maintain_quadtree.rs` —
  syncs `Aabb2D` entities into `CollisionQuadtree` using `GlobalPosition2D` as
  center; this is what cells register through

- `/Users/bgardner/dev/brickbreaker/breaker-game/src/state/run/node/systems/spawn_cells_from_layout/system.rs` —
  spawns cells; `Aabb2D::new(Vec2::ZERO, Vec2::new(cell_width / 2.0, cell_height / 2.0))`
  where `cell_width/height` are already multiplied by `compute_grid_scale`

- `/Users/bgardner/dev/brickbreaker/breaker-game/src/state/run/node/systems/apply_node_scale_to_bolt.rs` —
  inserts `NodeScalingFactor(layout.entity_scale)` on bolt entities; runs
  `OnEnter(NodeState::Loading)`

- `/Users/bgardner/dev/brickbreaker/breaker-game/src/bolt/plugin.rs` —
  `bolt_cell_collision` scheduling: after `MaintainQuadtree`,
  `EnforceDistanceConstraints`, `BreakerSystems::Move`

- `/Users/bgardner/dev/brickbreaker/breaker-game/src/bolt/systems/bolt_cell_collision/tests/aabb_collision.rs` —
  tests proving that `Aabb2D.half_extents` (not `CellWidth`/`CellHeight`) is
  the authoritative collision shape, and that `NodeScalingFactor` changes the
  effective bolt radius

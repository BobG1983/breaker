---
name: collision-system-map
description: Complete map of all collision detection systems in breaker-game — file paths, messages fired, detection logic for every pairing
type: reference
---

# Collision System Map (Bevy 0.18.1)

## Layer Constants (`src/shared/collision_layers.rs`)

| Layer | Constant | Value |
|-------|----------|-------|
| Bolt | `BOLT_LAYER` | `0x01` |
| Cell | `CELL_LAYER` | `0x02` |
| Wall | `WALL_LAYER` | `0x04` |
| Breaker | `BREAKER_LAYER` | `0x08` |

Bolt entity spawns with: `CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER)`

---

## Collision Systems

### 1. `bolt_cell_collision` — bolt vs cell AND bolt vs wall (physics CCD)

**File:** `src/bolt/systems/bolt_cell_collision/system.rs`
**Schedule:** `FixedUpdate`, run_if `PlayingState::Active`
**Set:** `BoltSystems::CellCollision`
**Ordering:** after `BoltSystems::PrepareVelocity`, after `PhysicsSystems::MaintainQuadtree`

**Detection method:** Swept circle CCD via `CollisionQuadtree::cast_circle`. Loops up to `MAX_BOUNCES = 4` per bolt per frame. Uses `CollisionLayers::new(0, CELL_LAYER | WALL_LAYER)` for the sweep — handles BOTH cell and wall hits in a single unified loop.

**Cell hit logic (CELL_LAYER only — no wall handling here):**
- Uses `CollisionLayers::new(0, CELL_LAYER)` — sweep is cells only, walls are NOT handled here.
- If bolt has `PiercingRemaining > 0` AND cell HP <= effective_damage → PIERCE: decrement PiercingRemaining, continue without reflecting
- Otherwise → NORMAL: reflect velocity off hit normal
- Always sends `BoltImpactCell { cell, bolt }` and `DamageCell { cell, damage, source_chip }`
- `effective_damage = BASE_BOLT_DAMAGE * (1.0 + DamageBoost)`

**Wall hit logic:** moved to `bolt_wall_collision` (see system 2 below).

**Messages fired:**
- `BoltImpactCell { cell: Entity, bolt: Entity }` — consumed by chips, cells, audio
- `DamageCell { cell: Entity, damage: f32, source_chip: Option<String> }` — consumed by `handle_cell_hit`

---

### 2. `bolt_wall_collision` — REAL IMPLEMENTATION (corrected 2026-03-30)

**File:** `src/bolt/systems/bolt_wall_collision/system.rs`
**Schedule:** `FixedUpdate`, run_if `PlayingState::Active`
**Ordering:** after `BoltSystems::CellCollision`

Fully implemented overlap resolver. After `bolt_cell_collision` moves the bolt via CCD,
this system checks whether the bolt center lies inside any wall's `Aabb2D` expanded by
`bolt_radius * EntityScale`. On overlap: finds nearest face, pushes bolt to face boundary,
reflects velocity off that face, resets `PiercingRemaining` to `EffectivePiercing.0`.
Only resolves the first wall overlap per bolt per frame (breaks after first hit).

**Messages fired:** `BoltImpactWall { bolt: Entity, wall: Entity }`

Note: `bolt_cell_collision` CCD sweep uses `CELL_LAYER` only — it does NOT handle wall
collision physics. Wall collisions are fully handled here.

---

### 3. `bolt_breaker_collision` — bolt vs breaker

**File:** `src/bolt/systems/bolt_breaker_collision/system.rs`
**Schedule:** `FixedUpdate`, run_if `PlayingState::Active`
**Set:** `BoltSystems::BreakerCollision`
**Ordering:** after `BoltSystems::CellCollision`

**Detection method:** Two-phase:
1. **Overlap resolution:** If bolt center is inside expanded AABB (half_w+r, half_h+r), push bolt above breaker and reflect if moving downward. Handles breaker-moving-into-bolt case (bump pop) that CCD can't detect.
2. **CCD sweep:** `Aabb2D::ray_intersect` from bolt position in velocity direction for `speed * dt` distance.

**Reflection logic (`reflect_top_hit`):**
- `hit_fraction = (impact_x - breaker_x) / half_w` clamped to [-1, 1]
- `base_angle = hit_fraction * max_angle`
- `total_angle = base_angle + tilt_angle` (breaker tilt added)
- Clamped to `[-effective_max_angle, effective_max_angle]`
- Speed enforced to `max(current_speed, base_speed)`
- `enforce_min_angle` applied to prevent near-horizontal shots

Side hit (normal.x > normal.y): reflects X only, preserves Y.
Top/bottom hit (normal.y >= normal.x): full angle reflection via `reflect_top_hit`.
Upward-moving bolts pass through on all faces.

**Resets PiercingRemaining to Piercing.0 on breaker hit.**

**Messages fired:**
- `BoltImpactBreaker { bolt: Entity }` — consumed by `grade_bump` (breaker domain)

---

### 4. `detect_breaker_cell_collision` — breaker vs cell

**File:** `src/breaker/systems/detect_breaker_cell_collision.rs`
**Schedule:** `FixedUpdate`, run_if `PlayingState::Active`
**Ordering:** after `BreakerSystems::Move`

**Detection method:** Simple AABB overlap: `dx < half_w + cell_half.x && dy < half_h + cell_half.y`. Checks every cell entity against the single breaker entity.

Note: Described as "currently a placeholder" — future moving-cell mechanics will make this active in gameplay.

**Messages fired:**
- `BreakerImpactCell { breaker: Entity, cell: Entity }` — consumed by effect bridges

---

### 5. `detect_breaker_wall_collision` — breaker vs wall

**File:** `src/breaker/systems/detect_breaker_wall_collision.rs`
**Schedule:** `FixedUpdate`, run_if `PlayingState::Active`
**Ordering:** after `BreakerSystems::Move`

**Detection method:** Simple AABB overlap: `dx < half_w + wall_half.x && dy < half_h + wall_half.y`. Note: `move_breaker` already clamps breaker to playfield bounds, so this catches edge-case overlaps for effect trigger chains.

**Messages fired:**
- `BreakerImpactWall { breaker: Entity, wall: Entity }` — consumed by effect bridges

---

### 6. `detect_cell_wall_collision` — cell vs wall

**File:** `src/cells/systems/detect_cell_wall_collision.rs`
**Schedule:** `FixedUpdate`, run_if `PlayingState::Active`
**Ordering:** no explicit constraint

**Detection method:** Simple AABB overlap: `dx < cell_half.x + wall_half.x && dy < cell_half.y + wall_half.y`. O(cells * walls). Described as "currently a placeholder" — for future moving-cell mechanics.

**Messages fired:**
- `CellImpactWall { cell: Entity, wall: Entity }` — consumed by effect bridges

---

### 7. `bolt_lost` — bolt falls below playfield (not collision, but boundary)

**File:** `src/bolt/systems/bolt_lost/system.rs`
**Schedule:** `FixedUpdate`, run_if `PlayingState::Active`
**Set:** `BoltSystems::BoltLost`
**Ordering:** after `PhysicsSystems::EnforceDistanceConstraints`, after `clamp_bolt_to_playfield`

**Detection:** `bolt_pos.y < playfield.bottom() - r`

**Behavior:**
- Baseline bolts (no `ExtraBolt`): respawned above breaker with random angle spread
- Extra bolts (`ExtraBolt` marker): sends `RequestBoltDestroyed { bolt }` for two-phase destruction

**Messages fired:**
- `BoltLost` — always (consumed by breaker plugin: applies penalty)
- `RequestBoltDestroyed { bolt: Entity }` — extra bolts only (consumed by `bridge_bolt_death` and `cleanup_destroyed_bolts`)

---

### 8. `handle_cell_hit` — damage consumer (not collision detection)

**File:** `src/cells/systems/handle_cell_hit/system.rs`
**Schedule:** `FixedUpdate`, run_if `PlayingState::Active`
**Ordering:** no explicit ordering constraint (before `check_lock_release`)

Consumes `DamageCell` messages. Not a collision detector — it applies damage and triggers destruction.
Deduplicates multiple hits on same cell in one frame via `Local<Vec<Entity>>`.
Locked cells immune to damage.

**Messages fired:**
- `RequestCellDestroyed { cell: Entity }` — when HP reaches 0 (consumed by `bridge_cell_death` and `cleanup_destroyed_cells`)

---

## Collision Pairings Summary

| Pairing | System | Method | Messages |
|---------|--------|--------|---------|
| Bolt → Cell | `bolt_cell_collision` | CCD quadtree sweep | `BoltImpactCell`, `DamageCell` |
| Bolt → Wall | `bolt_cell_collision` (unified) | CCD quadtree sweep | `BoltImpactWall` |
| Bolt → Breaker | `bolt_breaker_collision` | CCD ray + overlap resolve | `BoltImpactBreaker` |
| Breaker → Cell | `detect_breaker_cell_collision` | AABB overlap | `BreakerImpactCell` |
| Breaker → Wall | `detect_breaker_wall_collision` | AABB overlap | `BreakerImpactWall` |
| Cell → Wall | `detect_cell_wall_collision` | AABB overlap | `CellImpactWall` |
| Bolt lost | `bolt_lost` | Y-position boundary | `BoltLost`, `RequestBoltDestroyed` |

## Architectural Notes (updated 2026-03-30)

**Bolt-wall collision**: `bolt_wall_collision` is a real, fully implemented overlap resolver
(not a stub). It runs after `bolt_cell_collision`, handles push-out and reflection for walls,
and sends `BoltImpactWall`. `bolt_cell_collision` only sweeps CELL_LAYER.

**Breaker AABB in quadtree**: The stored `Aabb2D` on the breaker is NOT updated when
`EntityScale` changes. All breaker collision systems (`breaker_cell_collision`,
`breaker_wall_collision`, `bolt_breaker_collision`) compute scaled dimensions at runtime
from `BreakerWidth * EntityScale` rather than reading the stored `Aabb2D`. The stored
`Aabb2D` is only used for the quadtree's own broad-phase — which may be under-sized when
`EntityScale > 1.0`. `bolt_breaker_collision` has a fallback `ray_intersect` path that
compensates.

**Cell Aabb2D**: baked at spawn from pre-computed `cell_width/2, cell_height/2`. No
`EntityScale` on cells. Never changes after spawn.

The three AABB-overlap systems (breaker-cell, breaker-wall, cell-wall) are all real and
active. They are not "just placeholders" — they drive the `Impact`/`Impacted` effect trigger
chains. Cells are stationary so `cell_wall_collision` rarely fires in normal gameplay but
is architecturally live.

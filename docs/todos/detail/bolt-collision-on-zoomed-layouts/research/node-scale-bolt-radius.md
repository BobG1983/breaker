## Behavior Trace: node_scale → Bolt Visual Size and Collision Geometry

Bevy version: **0.18** (workspace `Cargo.toml`)

---

### Trigger

`OnEnter(NodeState::Loading)` — a new node starts loading.

---

### 1. Where does `entity_scale` come from?

`NodeLayout` (deserialized from `assets/nodes/*.node.ron`) carries an
`entity_scale: f32` field. It defaults to `1.0` via serde when absent.
The valid range is `0.5..=1.0` (enforced in `NodeLayout::validate`).

```rust
// breaker-game/src/state/run/node/definition/types.rs
pub struct NodeLayout {
    // ...
    #[serde(default = "default_entity_scale")]
    pub entity_scale: f32,
}
```

`set_active_layout` (runs `OnEnter(NodeState::Loading)`) picks the correct
`NodeLayout` from the `NodeLayoutRegistry` (or a `ScenarioLayoutOverride`) and
inserts it as the `ActiveNodeLayout` resource.

---

### 2. How does `entity_scale` reach the bolt? — `apply_node_scale_to_bolt`

File: `breaker-game/src/state/run/node/systems/apply_node_scale_to_bolt.rs`

Registered in: `BoltPlugin::build`, `OnEnter(NodeState::Loading)`,
after `NodeSystems::Spawn` (so cells exist before this runs).

```rust
pub(crate) fn apply_node_scale_to_bolt(
    layout: Option<Res<ActiveNodeLayout>>,
    query: Query<Entity, With<Bolt>>,
    mut commands: Commands,
) {
    let Some(layout) = layout else { return };
    for entity in &query {
        commands
            .entity(entity)
            .insert(NodeScalingFactor(layout.0.entity_scale));
    }
}
```

**Result**: Every bolt entity receives `NodeScalingFactor(entity_scale)` on the
frame `NodeState::Loading` is entered. Because `Commands` are deferred, the
component is inserted at the next command flush — but still within the same
`OnEnter` schedule before `FixedUpdate` begins.

`NodeScalingFactor` is defined in `breaker-game/src/shared/components.rs`:

```rust
/// Scale factor applied to breaker and bolt dimensions per layout.
/// Multiplies visual size and collision hitboxes — speed is unaffected.
#[derive(Component, Debug, Clone, Copy)]
pub struct NodeScalingFactor(pub f32);
```

---

### 3. How does `sync_bolt_scale` work?

File: `breaker-game/src/bolt/systems/sync_bolt_scale.rs`

Registered in: `BoltPlugin::build`, `Update` schedule,
`run_if(in_state(NodeState::Playing))`.

```rust
pub(crate) fn sync_bolt_scale(mut query: Query<SyncBoltScaleData, With<Bolt>>) {
    for mut data in &mut query {
        let boost = data.size_boosts.map_or(1.0, ActiveSizeBoosts::multiplier);
        let node  = data.node_scale.map_or(1.0, |s| s.0);
        let range = ClampRange {
            min: data.min_radius.map(|m| m.0),
            max: data.max_radius.map(|m| m.0),
        };
        let r = effective_radius(data.base_radius.0, boost, node, range);
        data.scale.x = r;
        data.scale.y = r;
    }
}
```

`effective_radius` (in `shared/size/types.rs`) is:

```rust
pub(crate) fn effective_radius(
    base_radius: f32,
    size_boost_multiplier: f32,
    node_scaling_factor: f32,
    radius_range: ClampRange,
) -> f32 {
    radius_range.apply(base_radius * size_boost_multiplier * node_scaling_factor)
}
```

**What `sync_bolt_scale` reads:**
- `BaseRadius` (`shared::size`) — the unscaled bolt radius (set at spawn from the
  bolt definition)
- `Scale2D` (`rantzsoft_spatial2d`) — mutable, this is what it writes
- `NodeScalingFactor` (`shared::components`) — optional; if absent, defaults to 1.0
- `ActiveSizeBoosts` — optional; bolt size upgrade stacking multiplier
- `MinRadius` / `MaxRadius` — optional hard caps

**What `sync_bolt_scale` writes:**
- `Scale2D.x = r` and `Scale2D.y = r` where `r = base_radius * boost * node_scale`
  (clamped to min/max if those components are present)

**What `sync_bolt_scale` does NOT touch:**
- `Aabb2D` — never read, never written by this system
- `BaseRadius` — read-only, never modified

**Visual path from `Scale2D` to screen:**

`compute_globals` (spatial plugin, `FixedUpdate`) propagates `Scale2D` →
`GlobalScale2D` for root entities (bolt is always a root entity).
`derive_transform` (spatial plugin, runs in `PostUpdate` after Bevy's
`TransformPropagate`) writes `GlobalScale2D` into `Transform.scale`.
Bevy's renderer reads `Transform` and `GlobalTransform`. The bolt mesh is a unit
circle of radius 1.0; `Transform.scale.xy = (r, r)` makes it appear as a circle
of radius `r` world units.

**Conclusion: `sync_bolt_scale` correctly scales the visual.** A bolt with
`BaseRadius(8.0)` and `NodeScalingFactor(0.5)` will visually render as a circle
of radius 4.0.

---

### 4. Does `sync_bolt_scale` update `Aabb2D`?

**No. `sync_bolt_scale` never touches `Aabb2D`.**

`Aabb2D` on the bolt is set once at spawn time by `build_core` in
`breaker-game/src/bolt/builder/core/terminal.rs`:

```rust
Aabb2D::new(Vec2::ZERO, Vec2::new(radius, radius)),
```

where `radius` is the unscaled bolt radius from the definition (e.g. 8.0 for
the default bolt). `NodeScalingFactor` is not yet available at spawn time
(it is inserted by `apply_node_scale_to_bolt` later in `OnEnter`).

No other system updates `Aabb2D.half_extents` on the bolt entity after spawn.

**However, this is not a collision bug for bolt-cell collision.** See section 5.

---

### 5. Does the bolt's `Aabb2D` affect bolt-cell collision geometry?

**No. `bolt_cell_collision` does not read the bolt's own `Aabb2D` at all.**

The bolt-cell collision system (`bolt_cell_collision`,
`bolt_wall_collision`, `bolt_breaker_collision`) computes the effective
bolt radius directly from components:

```rust
// bolt_cell_collision/system.rs
let bolt_scale = bolt.collision.node_scale.map_or(1.0, |s| s.0);
let r = bolt.collision.radius.0 * bolt_scale;
```

where `bolt.collision.radius` is `BoltRadius` (a type alias for `BaseRadius`)
and `bolt.collision.node_scale` is the `NodeScalingFactor` component.

This `r` is passed to `quadtree.cast_circle(position, direction, remaining, r, ...)`,
which performs a swept-circle CCD test. The bolt's own `Aabb2D` is irrelevant to
this computation.

The same pattern applies to `bolt_wall_collision` and `bolt_breaker_collision`:
both compute `r = radius.0 * bolt_scale` from the same components.

**So for bolt-cell collision, `Scale2D` and `Aabb2D` on the bolt entity are
both irrelevant.** The collision radius is always derived on-the-fly from
`BaseRadius * NodeScalingFactor`.

---

### 6. What IS the bolt's `Aabb2D` used for?

The bolt's `Aabb2D` is used by `maintain_quadtree` to register the bolt in the
`CollisionQuadtree`. Other systems that query the quadtree for bolts by spatial
region use the stored entry. Examples: any system doing broad-phase detection
of "which bolts are near X" would use this. Since `bolt_cell_collision` casts
from the bolt's position outward, it does not need the bolt's own quadtree entry.

Because `maintain_quadtree` updates the quadtree entry when
`Changed<GlobalPosition2D>` fires, the bolt's entry in the quadtree follows its
position correctly. But `half_extents` on the stored AABB is always the original
unscaled `radius`, not the node-scaled radius. This is a latent inaccuracy for
any future system that queries the quadtree to find bolts by area and relies on
the stored half_extents being accurate to the actual bolt size.

---

### System Chain Summary

```
OnEnter(NodeState::Loading):
  set_active_layout                  → ActiveNodeLayout(NodeLayout { entity_scale })
  apply_node_scale_to_bolt           → NodeScalingFactor(entity_scale) on all Bolt entities
                                        (Commands — deferred, applied same schedule)

Every Update frame (NodeState::Playing):
  sync_bolt_scale                    → Scale2D = BaseRadius * boost * NodeScalingFactor (clamped)

  compute_globals (spatial, FixedUpdate) → GlobalScale2D.x/y = Scale2D.x/y (for root entities)

  derive_transform (spatial, PostUpdate) → Transform.scale = GlobalScale2D
                                           Renderer reads Transform → visual size on screen

Every FixedUpdate frame (NodeState::Playing):
  maintain_quadtree                  → Bolt entity's Aabb2D (half = base_radius, UNSCALED)
                                        registered at GlobalPosition2D

  bolt_cell_collision                → r = BoltRadius * NodeScalingFactor (computed live)
                                        cast_circle(pos, dir, dist, r, CELL_LAYER)
                                        Minkowski expansion: cell AABB + r circle
                                        Result: bolt CCD uses SCALED radius
```

---

### Gap Analysis

**Visual — no gap.** `sync_bolt_scale` correctly applies `NodeScalingFactor` to
`Scale2D`, which propagates through the spatial pipeline to `Transform.scale`.
The bolt renders at the correct scaled size.

**Bolt-cell/wall/breaker collision — no gap.** All three collision systems
compute `r = BaseRadius * NodeScalingFactor` live, bypassing `Aabb2D`
entirely. The scaled radius is used for all CCD intersection tests.

**Bolt's own `Aabb2D` — latent inaccuracy, no current impact.** The bolt's
`Aabb2D.half_extents` is set to `base_radius` at spawn and never updated to
reflect `NodeScalingFactor`. The bolt's quadtree entry therefore stores the
wrong half_extents when `entity_scale < 1.0`. No current system relies on the
bolt's quadtree entry for collision geometry, so this does not produce incorrect
behavior today. If a future system queries the quadtree to find bolts by area
and expects the stored AABB to reflect actual bolt size, it will get incorrect
results.

**`bolt_lost` detection — consistent with collision.** `bolt_lost/system.rs`
also computes `r = bolt.radius.0 * bolt.node_scale.map_or(1.0, |s| s.0)` before
testing `position.y < playfield.bottom() - r`. The loss boundary correctly
accounts for the scaled bolt radius.

---

### Edge Cases

1. **Bolt spawned before `apply_node_scale_to_bolt` runs**: The bolt builder
   does not insert `NodeScalingFactor`. If a bolt is somehow queried before
   `OnEnter(NodeState::Loading)` runs `apply_node_scale_to_bolt`, the
   `node_scale` field in all collision systems `map_or(1.0, ...)` fallback means
   the unscaled `BaseRadius` is used. Bolt spawns at `OnEnter(NodeState::Loading)`
   via `reset_bolt`, which runs in the same schedule — ordering is
   `apply_node_scale_to_bolt.after(NodeSystems::Spawn)`, so cells are spawned
   before scale is applied, but `reset_bolt` also runs `after(BreakerSystems::Reset)`.
   The commands that insert `NodeScalingFactor` are deferred, so on the first
   `FixedUpdate` tick the component should already be present.

2. **`sync_bolt_scale` runs in `Update`, collision in `FixedUpdate`**: They run
   in different schedules. `Scale2D` (visual) is updated every `Update` frame,
   while collision uses `NodeScalingFactor` directly every `FixedUpdate` frame.
   Both read from the same `NodeScalingFactor` component, so they are consistent
   — a change to `NodeScalingFactor` (which only happens at `OnEnter`) will be
   reflected in both visual and collision on the same logical frame.

3. **`NodeScalingFactor` is overwrote on node transition**: `apply_node_scale_to_bolt`
   uses `commands.entity(entity).insert(...)` which overwrites any existing value.
   This is correct behavior for node transitions where a new layout with a
   different `entity_scale` is loaded.

4. **Extra bolts (spawned by effects, not at node start)**: `apply_node_scale_to_bolt`
   queries `With<Bolt>`. Extra bolts spawned after `OnEnter(NodeState::Loading)`
   (e.g., by `spawn_bolts` effect during gameplay) will NOT have
   `NodeScalingFactor` inserted by `apply_node_scale_to_bolt` — that system only
   runs once per node entry. The collision fallback `map_or(1.0, ...)` means
   extra bolts use unscaled radius. Whether this is correct depends on the design
   intent — if extra bolts should also be scaled, they need `NodeScalingFactor`
   injected at spawn time.

---

### Key Files

- `breaker-game/src/state/run/node/definition/types.rs` — `NodeLayout.entity_scale`
  field definition, default 1.0, valid range 0.5..=1.0

- `breaker-game/src/state/run/node/systems/apply_node_scale_to_bolt.rs` —
  inserts `NodeScalingFactor(entity_scale)` on all bolt entities;
  runs `OnEnter(NodeState::Loading)` via `BoltPlugin`

- `breaker-game/src/bolt/systems/sync_bolt_scale.rs` — computes
  `Scale2D = effective_radius(BaseRadius, boost, NodeScalingFactor, clamp)`;
  runs `Update` while `NodeState::Playing`; does NOT touch `Aabb2D`

- `breaker-game/src/shared/size/types.rs` — `effective_radius` function:
  `base * boost * node_scale` then clamped

- `breaker-game/src/bolt/builder/core/terminal.rs` — `build_core`:
  sets `Aabb2D::new(Vec2::ZERO, Vec2::new(radius, radius))` at spawn (unscaled,
  never updated)

- `breaker-game/src/bolt/systems/bolt_cell_collision/system.rs` — line 112-113:
  `r = bolt.collision.radius.0 * bolt_scale` — live computation from
  `BoltRadius * NodeScalingFactor`; `Aabb2D` on bolt is NOT read here

- `breaker-game/src/bolt/systems/bolt_wall_collision/system.rs` — same pattern,
  line 45-46

- `breaker-game/src/bolt/systems/bolt_breaker_collision/system.rs` — same
  pattern, line 306-307

- `breaker-game/src/bolt/systems/bolt_lost/system.rs` — line 68:
  same pattern for loss boundary detection

- `breaker-game/src/bolt/queries.rs` — `BoltCollisionParams` struct: includes
  `radius: &'static BoltRadius` and `node_scale: Option<&'static NodeScalingFactor>`

- `rantzsoft_physics2d/src/systems/maintain_quadtree.rs` — syncs `Aabb2D`
  (with its original half_extents) into the quadtree; watches
  `Changed<GlobalPosition2D>` but not `Changed<Aabb2D>` or `Changed<Scale2D>`

- `rantzsoft_spatial2d/src/systems/compute_globals/system.rs` — propagates
  `Scale2D` → `GlobalScale2D`

- `rantzsoft_spatial2d/src/systems/derive_transform.rs` — writes
  `GlobalScale2D` → `Transform.scale` for visual rendering

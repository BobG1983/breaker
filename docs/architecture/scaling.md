# Entity Scale

Per-node scaling of breaker and bolt via `NodeScalingFactor`. This is the architecture reference for how scaling is implemented. For the design rationale (why speed is constant, recommended ranges, pillar alignment), see `docs/design/decisions/entity-scale.md`.

## Components

| Component | Location | Description |
|---|---|---|
| `NodeScalingFactor(f32)` | `shared/components.rs` | Per-node scale factor (0.5–1.0). Defaults to 1.0. |
| `BaseWidth(f32)` | `shared/components.rs` | Unscaled width. Set by builder from definition. |
| `BaseHeight(f32)` | `shared/components.rs` | Unscaled height. Set by builder from definition. |
| `BaseRadius(f32)` | `shared/size/types.rs` | Unscaled bolt radius. Aliased as `BoltRadius` in the bolt domain. |

All four are simple newtype components. `NodeScalingFactor` is inserted as a `Resource`-like shared value; `BaseWidth`/`BaseHeight`/`BaseRadius` are per-entity components set by builders.

## Scaling Formula

Entity scale applies as a final multiplier on the total (base + boost):

```
effective_width  = (base_width + width_boost) * node_scaling_factor
effective_height = (base_height + height_boost) * node_scaling_factor
effective_radius = base_radius * node_scaling_factor
```

Both visual size (`Transform.scale`) AND collision hitboxes (`Aabb2D`) scale together — no visual-only tricks.

## Application Systems

The scaling is applied by `apply_node_scale_to_*` systems registered per entity domain:

- `apply_node_scale_to_bolt` — in `bolt/`, runs `.after(spawn_bolt).after(NodeSystems::Spawn)`
- `apply_node_scale_to_breaker` — in `breaker/`, runs `.after(spawn_or_reuse_breaker)`
- `apply_node_scale_to_cells` — in `cells/`, runs `.after(NodeSystems::Spawn)`

Each system reads `NodeScalingFactor`, multiplies against the entity's base size, and updates `Transform.scale` + `Aabb2D` accordingly.

## What is NOT scaled

Bolt speed and breaker movement speed are **not** affected by `NodeScalingFactor`. This is the central design rule: smaller hitboxes at the same speed = tighter gameplay. The formula above affects only spatial dimensions, never velocity.

## Source

`NodeScalingFactor` value comes from the node layout RON file's `entity_scale` field (optional, defaults to 1.0). It is set as a shared resource at the start of each node by the node initialization system.

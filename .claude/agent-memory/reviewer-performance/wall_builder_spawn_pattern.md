---
name: WallBuilder spawn pattern
description: WallBuilder typestate builder for wall entities — spawn-time only, ~3-4 calls at node start
type: project
---

Wall entities are built via `Wall::builder()` → side transition → optional `.definition()` / `.with_*()` chainables → `.visible()` or `.invisible()` → `.spawn()` or `.build()`.

Reviewed files:
- `src/walls/builder/core/types.rs` — typestate markers, OptionalWallData, WallBuilder struct
- `src/walls/builder/core/transitions.rs` — entry point, side transitions, definition(), with_* chainables, visible()
- `src/walls/builder/core/terminal.rs` — build_core(), resolve_half_thickness(), resolve_effects(), dispatch_effects(), build/spawn impls
- `src/walls/definition.rs` — WallDefinition (name, half_thickness, color_rgb, Vec<RootNode>)
- `src/state/run/node/systems/spawn_walls/system.rs` — spawn_walls system; uses Wall::builder() for left/right/ceiling walls

## Confirmed acceptable patterns

- All builder calls are spawn-time only (~3 calls at node start for permanent walls + 1 for SecondWind floor). Zero per-frame cost.
- No queries, no systems, no scheduling concerns in this builder.
- `OptionalWallData` stores `Option<Vec<RootNode>>` for definition and override effects separately. `Clone` via `.definition(def)` clones `def.effects` — acceptable at spawn-time.
- `resolve_effects()` clones the `Option<Vec<RootNode>>` out of OptionalWallData — one clone at spawn per wall with effects. Normal walls (standard left/right/ceiling) have no effects, so `resolve_effects()` returns `None` and no clone occurs.
- `dispatch_effects()` in terminal.rs allocates a `Vec<(String, Tree)>` only when effects are present. Standard walls have no effects. For effect walls: one allocation at spawn-time, acceptable.
- `String::new()` (empty string) is allocated per effect entry in `dispatch_effects` — this is the chip-name slot for wall-bound effects; not a hot path.
- `visible()` transition creates `Handle<Mesh>` + `Handle<ColorMaterial>` by calling `meshes.add(Rectangle::new(1.0, 1.0))` and `materials.add(...)` — one mesh allocation per visible wall at spawn. No per-frame concern.
- `build_core()` returns a stack-allocated tuple bundle — no heap allocation.
- Archetype impact: invisible walls produce one archetype (Wall + Position2D + Scale2D + Aabb2D + CollisionLayers + GameDrawLayer). Visible walls add Mesh2d + MeshMaterial2d → separate archetype. Effect walls add BoundEffects + StagedEffects → additional archetype split. At 4 walls this is negligible.

## spawn_walls system
`spawn_walls` uses `Wall::builder()` for left/right/ceiling walls. SecondWind floor wall also uses the builder via `second_wind/system.rs`. No performance issue — all spawn-time only.

**Why:** 4 walls, spawned once at node start. Builder pattern adds zero runtime overhead beyond the direct spawn path.

**How to apply:** Do not flag builder chain allocation patterns for walls. Only raise concerns if walls become per-frame entities or counts scale beyond tens.

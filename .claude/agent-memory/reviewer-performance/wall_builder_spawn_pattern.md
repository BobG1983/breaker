---
name: WallBuilder spawn pattern
description: WallBuilder typestate builder for wall entities — spawn-time only, ~3-4 calls at node start
type: project
---

Wall entities are built via `Wall::builder()` → side transition → optional `.definition()` / `.with_*()` chainables → `.visible()` or `.invisible()` → `.spawn()` or `.build()`.

Reviewed files:
- `src/wall/builder/core/types.rs` — typestate markers, OptionalWallData, WallBuilder struct
- `src/wall/builder/core/transitions.rs` — entry point, side transitions, definition(), with_* chainables, visible()
- `src/wall/builder/core/terminal.rs` — build_core(), resolve_half_thickness(), resolve_effects(), dispatch_effects(), build/spawn impls
- `src/wall/definition.rs` — WallDefinition (name, half_thickness, color_rgb, Vec<RootEffect>)
- `src/wall/systems/spawn_walls/system.rs` — legacy spawn_walls system (manual spawn, no builder)

## Confirmed acceptable patterns

- All builder calls are spawn-time only (~3 calls at node start for permanent walls + 1 for SecondWind floor). Zero per-frame cost.
- No queries, no systems, no scheduling concerns in this builder.
- `OptionalWallData` stores `Option<Vec<RootEffect>>` for definition and override effects separately. `Clone` via `.definition(def)` clones `def.effects` — acceptable at spawn-time.
- `resolve_effects()` clones the `Option<Vec<RootEffect>>` out of OptionalWallData — one clone at spawn per wall with effects. Normal walls (standard left/right/ceiling) have no effects, so `resolve_effects()` returns `None` and no clone occurs.
- `dispatch_effects()` in terminal.rs allocates a `Vec<(String, EffectNode)>` only when effects are present. Standard walls have no effects. For effect walls: one allocation at spawn-time, acceptable.
- `String::new()` (empty string) is allocated per effect entry in `dispatch_effects` — this is the chip-name slot for wall-bound effects; not a hot path.
- `visible()` transition creates `Handle<Mesh>` + `Handle<ColorMaterial>` by calling `meshes.add(Rectangle::new(1.0, 1.0))` and `materials.add(...)` — one mesh allocation per visible wall at spawn. No per-frame concern.
- `build_core()` returns a stack-allocated tuple bundle — no heap allocation.
- Archetype impact: invisible walls produce one archetype (Wall + Position2D + Scale2D + Aabb2D + CollisionLayers + GameDrawLayer). Visible walls add Mesh2d + MeshMaterial2d → separate archetype. Effect walls add BoundEffects + StagedEffects → additional archetype split. At 4 walls this is negligible.

## Legacy spawn_walls system
The `spawn_walls` system still exists and uses manual `commands.spawn()` instead of the builder. It spawns left/right/ceiling at node start. The builder is the intended pattern going forward (used for SecondWind floor wall). No performance issue — both paths are spawn-time only.

**Why:** 4 walls, spawned once at node start. Builder pattern adds zero runtime overhead beyond the direct spawn path.

**How to apply:** Do not flag builder chain allocation patterns for walls. Only raise concerns if walls become per-frame entities or counts scale beyond tens.

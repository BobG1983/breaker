# Cell Builder

Typestate builder for cell entity construction. Follows the same pattern as [bolt](bolt.md) and [breaker](breaker.md) builders.

## Entry Point

```rust
Cell::builder()  // -> CellBuilder<NoPosition, NoDimensions, NoHealth, Unvisual>
```

## Typestate Dimensions

| Dimension | Unconfigured | Configured | Transition Method |
|-----------|-------------|------------|-------------------|
| Position | `NoPosition` | `HasPosition { pos: Vec2 }` | `.position(Vec2)` |
| Dimensions | `NoDimensions` | `HasDimensions { width, height }` | `.dimensions(w, h)` |
| Health | `NoHealth` | `HasHealth { hp: f32 }` | `.definition(&def)` (production); `.hp(value)` (test-only) |
| Visual | `Unvisual` | `Rendered` / `Headless` | `.rendered(meshes, materials)` / `.headless()` |

`.definition(&CellTypeDefinition)` transitions Health and populates optional data (alias, damage visuals, behaviors, effects, color) from the RON-loaded definition.

## Resolution Order

`override > definition > default`

- `.definition(&def)` stores definition-derived values
- Individual setters (`.override_hp()`, `.alias()`, etc.) override specific values
- Terminal `spawn()` resolves each field: override wins, then definition, then a sensible default

## Optional Methods (any typestate)

| Method | Effect |
|--------|--------|
| `.alias(String)` | Sets `CellTypeAlias` component |
| `.locked(Vec<Entity>)` | Inserts `LockCell` + `Locked` + `Locks` components |
| `.guarded(Vec<u8>, GuardianSpawnConfig)` | Spawns guardian children in ring slots |
| `.override_hp(f32)` | Overrides definition HP — only available after `HasHealth` is set (i.e., after `.definition()`) |

Test-only methods (`#[cfg(test)]`): `.hp()`, `.headless()`, `.required_to_clear()`, `.damage_visuals()`, `.with_effects()`, `.with_behavior()`, `.color_rgb()`. In production, `.rendered()` accepts `&mut Assets<Mesh>` and `&mut Assets<ColorMaterial>` and pre-computes guardian visual handles when `.guarded()` has been called.

## Terminal: `spawn()`

`spawn()` is the only public terminal. No `build()` is exposed.

1. Resolves all optional data (override > definition > default)
2. Spawns core entity: `Cell`, `Position2D`, `Scale2D`, `Aabb2D`, `CollisionLayers`, `Hp`, `CellWidth`, `CellHeight`
3. Inserts optional markers: `CellTypeAlias`, `RequiredToClear`, `CellDamageVisuals`
4. Inserts lock components if `.locked()` was called
5. Processes `CellBehavior` list from definition: `Regen` -> regen components, `Guarded` -> `GuardedCell` marker
6. Dispatches effect chains via `commands.stamp_effect` (one per `RootNode::Stamp` in the cell's effects list)
7. If `.guarded()` was called: inserts `GuardedCell` on parent, spawns guardian children with `ChildOf`
8. If rendered: adds `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Cell` (and same on guardians)

## Guardian Spawning

When `.guarded(slots, config)` is called:
- `slots: Vec<u8>` — ring slot indices (0-7) provided by the spawn pipeline from node layout data
- `config: GuardianSpawnConfig` — hp, color, speed, dimensions, step sizes

Guardian visual handles are pre-computed during `.rendered()` and stored in `GuardedSpawnData.guardian_visuals`. Each guardian gets: `Cell`, `GuardianCell`, `GuardianSlot`, `SlideTarget`, `GuardianSlideSpeed`, `GuardianGridStep`, `Hp`, square dimensions, `PositionPropagation::Absolute`, `ChildOf(parent)`.

## File Layout

```
cells/builder/
  mod.rs
  core/
    mod.rs
    types.rs        — CellBuilder<P,D,H,V>, OptionalCellData, GuardedSpawnData, GuardianSpawnConfig
    transitions.rs  — Cell::builder(), dimension transitions, optional methods
    terminal.rs     — spawn(), spawn_inner(), spawn_guardian_children()
  tests/
    mod.rs
    typestate_tests.rs
    build_tests.rs
    definition_tests.rs
    spawn_tests.rs
    optional_tests.rs
    integration_tests.rs
```

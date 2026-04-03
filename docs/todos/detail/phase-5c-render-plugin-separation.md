# 5c: Crate Setup + Plugin Separation

**Goal**: Create the `rantzsoft_vfx` crate, extract visual concerns from gameplay plugins, eliminate `ui/` and `fx/` domains, and wire up `RantzVfxPlugin` in `game.rs`.

## Current State

Visual code is scattered across gameplay domains:
- `fx/` owns transitions, fade-out animations, punch scale animations
- `ui/` owns chip select, menus, pause, HUD, side panels
- Entity spawning systems (bolt, breaker, cells) directly attach `Mesh2d`, `MeshMaterial2d`, and color components
- No dedicated rendering infrastructure exists

## What to Build

### 1. Create `rantzsoft_vfx` Crate

New workspace member at `rantzsoft_vfx/` following rantzsoft_* conventions:
- `RantzVfxPlugin` — main Bevy plugin with `default()` and `headless()` constructors
- Takes a `VfxLayer` trait impl for z-ordering (game provides its draw layer mapping)
- `VfxConfig` resource (crate defines type, game inserts and mutates)
- Zero game vocabulary — game-agnostic, usable by any 2D Bevy game
- See `docs/architecture/rendering/rantzsoft_vfx.md` for full scope

### 2. Stub Core Systems

Register the infrastructure that later steps will build on:
- Message types (AttachVisuals, ExecuteRecipe, SetModifier, AddModifier, RemoveModifier, all per-primitive messages)
- RecipeStore resource + SeedableRegistry loading pipeline
- ModifierStack component + modifier computation system (stub)
- Particle system infrastructure (emitter, update, cleanup systems)

### 3. Absorb fx/ and ui/ Domains

- Transitions → `screen/transition/`
- Fade-out, punch scale → `rantzsoft_vfx` (generic animation primitives)
- Per-screen UI → `screen/<screen_name>/`
- HUD → `screen/playing/hud/`
- Remove `FxPlugin` and `UiPlugin` from `game.rs`
- See `docs/architecture/rendering/communication.md` for the full domain restructuring

### 4. Wire Game Plugin

- Register `RantzVfxPlugin` in `game.rs`
- Implement `VfxLayer` on a game type for z-ordering
- Insert `VfxConfig` from `GraphicsDefaults` RON via `rantzsoft_defaults`
- Configure recipe asset path (`assets/recipes/*.recipe.ron`)

### 5. Extract Visual Spawning from Gameplay

Entity spawn systems currently create `Mesh2d`/`MeshMaterial2d` directly. Refactor to:
- Gameplay spawns entity with gameplay components only (no Mesh2d)
- Gameplay sends `AttachVisuals { entity, config }` message
- Crate handles mesh/material/shader attachment

This is a stub at this step — `AttachVisuals` handler is a no-op or minimal. Full entity visuals come in 5g-5j.

## What NOT to Do

- Do NOT implement shaders or materials yet — that's 5d+
- Do NOT implement entity visual rendering yet — that's 5g-5j
- Do NOT change visual appearance — the game should look identical after this step (visual spawning extraction can use passthrough to current Mesh2d approach temporarily)

## Dependencies

- None — this is the first implementation step

## Files Affected

- New: `rantzsoft_vfx/` crate (Cargo.toml, lib.rs, plugin.rs, types, messages, stubs)
- Modified: `Cargo.toml` (workspace member)
- Modified: `src/game.rs` (replace FxPlugin/UiPlugin with RantzVfxPlugin + ScreenPlugin changes)
- Modified: `screen/` (absorbs transition and per-screen UI code from fx/ and ui/)
- Removed: `src/fx/` (absorbed into screen/transition/ and rantzsoft_vfx)
- Removed: `src/ui/` (absorbed into screen/<screen_name>/)

## Verification

- Game compiles with the new crate
- All existing tests pass
- Game looks and plays identically (visual spawning passthrough)
- `RantzVfxPlugin::headless()` works in scenario runner
- Message types are registered (game code can send VFX messages without panics)

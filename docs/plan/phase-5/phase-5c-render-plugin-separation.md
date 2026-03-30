# 5c: Render Plugin Separation

**Goal**: Extract visual-only concerns from gameplay plugins and establish the `rendering/` domain as the single owner of all rendering code (shaders, materials, post-processing, particles, VFX).

## Current State

Visual code is scattered across gameplay domains:
- `fx/` owns transitions, fade-out animations, punch scale animations
- Entity spawning systems (bolt, breaker, cells) directly attach `Mesh2d`, `MeshMaterial2d`, and color components
- No dedicated rendering infrastructure exists

## What to Build

### 1. Create `rendering/` Domain

New domain at `src/rendering/` with standard plugin structure:
- `RenderingPlugin` — registered in `game.rs`
- Owns all shader/material definitions, post-processing systems, particle systems, VFX spawning
- Reads render state components and render event messages from other domains

### 2. Absorb fx/ into rendering/

Merge the current fx/ domain (transitions, fade-out, punch scale) into rendering/:
- `rendering/transition/` — existing transition code
- `rendering/animation/` — fade-out, punch scale, future animation systems
- Remove fx/ as a standalone domain
- Update `game.rs` plugin registration

### 3. Establish Render Communication Interfaces

Define the interface contracts (traits or component types) that gameplay domains will implement:

**Render state components** (defined in rendering/, used by gameplay domains):
- Marker trait or convention for `*RenderState` components
- These will be populated in later steps (5g-5j) when entity visuals are implemented

**Render event messages** (each VFX module defines its own):
- Module-owned message types: `SpawnShockwaveVfx`, `SpawnChainLightningVfx`, `PlayBumpFeedbackVfx`, etc.
- Standard Bevy messages (not observers) — systems run in parallel via the scheduler
- `VfxKind` enum for RON data authoring — dispatch system translates enum → module message
- Completion messages: module-owned (`ChainLightningVfxComplete`, etc.) — gameplay reads these

### 4. Extract Visual Spawning from Gameplay

Current entity spawn systems (e.g., `spawn_bolt`, `spawn_breaker`, `spawn_cells`) directly create meshes and materials. Refactor so that:
- Gameplay spawn systems create the entity with gameplay components + visual identity components
- rendering/ observes new entities (via `Added<BoltVisualIdentity>` etc.) and attaches visual components (mesh, material, shader)
- This decouples gameplay from rendering details

## What NOT to Do

- Do NOT implement any new shaders or materials yet — that's 5d+
- Do NOT implement render state sync yet — that's per-entity in 5g-5j
- Do NOT change any visual appearance — the game should look identical after this step
- Do NOT add audio hooks

## Dependencies

- None — this is the first Phase 5 step

## Files Affected

- New: `src/rendering/` (mod.rs, plugin.rs, messages.rs, animation/, transition/)
- Modified: `src/game.rs` (replace FxPlugin with RenderingPlugin)
- Modified: entity spawn systems in bolt/, breaker/, cells/ (extract mesh/material creation)
- Removed: `src/fx/` (absorbed into rendering/)

## Verification

- Game looks and plays identically to before
- All existing tests pass
- rendering/ is the only domain that creates Mesh2d/MeshMaterial2d components
- Gameplay domains have no direct rendering dependencies (no Mesh2d imports)

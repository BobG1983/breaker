# 5a: Rendering Architecture Document

**Goal**: Write `docs/architecture/rendering.md` — the complete architectural specification for the rendering/ domain — before any code is written. This document is the contract that all subsequent Phase 5 steps implement against.

## What to Produce

### `docs/architecture/rendering.md`

A comprehensive architecture document covering:

### 1. Domain Structure

- `rendering/` directory layout: `plugin.rs`, `mod.rs`, `messages.rs`, `vfx/`, `screen_effects/`, `animation/`, `transition/`, `materials/`, `particles/`
- How VFX modules are organized: `rendering/vfx/<effect_name>/` with `mod.rs`, `messages.rs`, `systems.rs`
- How screen effects are organized: `rendering/screen_effects/<effect_name>/`

### 2. Communication Patterns

**Gameplay → Rendering (continuous state)**:
- `*RenderState` component pattern — how domains expose rendering-relevant state
- Update frequency, what goes in vs. what doesn't
- Examples: `BoltRenderState`, `CellRenderState`, `BreakerRenderState`

**Gameplay → Rendering (identity)**:
- Visual identity components set at spawn — separate components (`Shape`, `Color`, `AuraType`, etc.)
- How rendering/ detects new entities (`Added<*>` queries)
- How rendering/ attaches visual components (mesh, material, shader)

**Gameplay → Rendering (events)**:
- Module-owned message pattern — each VFX module defines its own message type
- Standard Bevy messages (not observers) for parallel system execution
- `VfxKind` enum for RON data dispatch — how the effect/ domain translates enum → module message
- When to use the enum dispatch vs. direct module message

**Rendering → Gameplay (completion)**:
- Module-owned completion messages for sequencing (e.g., `ChainLightningVfxComplete`)
- When completion messages are needed vs. when fire-and-forget is sufficient

### 3. How-To Guides

Step-by-step instructions for common operations:

**Adding a new VFX module**:
1. Create `rendering/vfx/<name>/` directory
2. Define message type in `messages.rs`
3. Implement system in `systems.rs`
4. Register in `rendering/vfx/mod.rs`
5. Add variant to `VfxKind` if RON-authored
6. Add dispatch arm in effect/ domain if data-driven

**Adding a new screen effect**:
1. Create `rendering/screen_effects/<name>/`
2. Define message and system
3. Register in plugin

**Adding a new entity visual**:
1. Define visual identity component in owning domain
2. Define render state component in owning domain
3. Add rendering/ system that observes `Added<VisualIdentity>` and attaches visuals
4. Add rendering/ system that reads render state and updates visuals

### 4. Integration Points

- How rendering/ plugs into the existing plugin architecture
- System ordering: when do rendering systems run relative to gameplay?
- How the post-processing pipeline is structured (render graph nodes)
- How the particle system integrates

### 5. Conventions

- Naming conventions for messages, components, systems
- File size and splitting guidelines (consistent with existing architecture)
- What lives in rendering/ vs. what lives in the owning domain

## What NOT to Do

- Do NOT write any code — this is a documentation step
- Do NOT design specific VFX — that's steps 5g-5w

## Dependencies

- None — this is the first Phase 5 step

## Output

- `docs/architecture/rendering.md` — the authoritative rendering architecture spec
- Reviewed and approved before proceeding to 5b or 5c

## Verification

- Architecture doc is consistent with the communication patterns described in `phase-5/index.md`
- Architecture doc covers all patterns that later steps will need
- Architecture doc follows the style and depth of existing docs (e.g., `docs/architecture/effects.md`)

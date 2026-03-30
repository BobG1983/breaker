# Phase 5: Visual Identity

**Goal**: Establish the stylized, shader-driven aesthetic. Not polish — identity. Every entity, effect, and screen should communicate through light, geometry, and motion — no floating text, no damage numbers, no UI overlays where visual feedback serves better.

## Architecture Decision: rendering/ Domain

Phase 5 introduces a new `rendering/` domain that owns all visual rendering code (shaders, materials, post-processing, particles, VFX). Visual configuration data (shapes, colors, aura types) lives in the owning domain as components; rendering/ reads these through curated interfaces.

### Communication Pattern

| Direction | Mechanism | Examples |
|-----------|-----------|---------|
| Gameplay → Rendering (continuous) | 1+ `*RenderState` components on entities | `BoltRenderState { speed, direction }`, `CellRenderState { health_fraction }` |
| Gameplay → Rendering (identity) | Separate visual components set at spawn | `Shape`, `Color`, `AuraType`, `TrailType`, `DamageDisplay`, `DeathEffect` — entities get only the ones that apply |
| Gameplay → Rendering (events) | Module-owned messages (Bevy `Message`, not observers) | `SpawnShockwaveVfx { pos, radius }`, `PlayBumpFeedbackVfx { grade, pos }` |
| Rendering → Gameplay (completion) | Module-owned completion messages | `ChainLightningVfxComplete { .. }`, `TransitionAnimationComplete` |

rendering/ never imports gameplay internals directly — only reads curated render state components and dedicated render messages.

### Domain Ownership

| Concern | Owner | Example |
|---------|-------|---------|
| Visual config data | Owning domain | Definitions of what's available live in Rendering/, setup lives in Breaker/ Chip/ etc. |
| Render state sync | Owning domain | breaker/ maintains 1+ `RenderState` style components|
| Shader/material code | rendering/ | Bolt glow shader, cell shape meshes |
| Post-processing | rendering/ | Screen distortion, bloom tuning, chromatic aberration |
| Particles | rendering/ | All particle emitters and systems |
| VFX spawning | rendering/ | Observes render events, spawns VFX entities |
| Animation completion | rendering/ | Emits completion messages back to gameplay |

### Render Message Pattern

Each VFX module (`rendering/vfx/shockwave/`, `rendering/vfx/chain_lightning/`, etc.) defines its own message type and systems. A `VfxKind` enum exists for RON data authoring — a single dispatch system in the effect/ domain translates `VfxKind` → the correct module message when data-driven effects fire. Gameplay domains that know the specific effect can also send the module message directly.

- **Module-owned messages**: `SpawnShockwaveVfx`, `SpawnChainLightningVfx`, `PlayBumpFeedbackVfx`, etc.
- **Standard Bevy systems** (not observers): systems read messages and run in parallel via the scheduler
- **RON enum**: `VfxKind { Shockwave, ChainLightning, ... }` for data-driven dispatch only
- **Completion messages**: Module-owned (`ChainLightningVfxComplete`, etc.) — gameplay systems read these to sequence dependent behavior

### Architecture Documentation

`docs/architecture/rendering.md` must be written as part of step 5a, **before** any VFX implementation. It documents:
- How to create a new VFX module (directory structure, message type, system registration)
- How to create a new screen effect
- The render state component pattern
- The visual identity component pattern
- The RON `VfxKind` dispatch pattern

This follows the precedent set by `docs/architecture/effects.md` for the chip effect system.

## Design Decisions

Several open design decisions (DR-1 through DR-10 in `docs/design/graphics/decisions-required.md`) must be resolved before implementing the steps that depend on them. Step 5b is a dedicated decision resolution step.

## Audio

Phase 5 is purely visual. All audio work (SFX, music, heartbeat timer, layered intensity) belongs to Phase 6. VFX systems in Phase 5 do not emit audio events or include audio stubs.

## Subphases

Steps are ordered by dependency. Architecture and decisions (5a-5b) come first. Infrastructure steps (5c-5f) establish foundations. Entity visuals (5g-5j) build the core look. Effects and feedback (5k-5o) add juice. UI and screens (5p-5s) complete the experience. Evolution VFX (5t-5w) are the crown jewels.

Steps 5d and 5e are independent and can be done in either order.

### Architecture & Decisions

- [5a: Rendering Architecture](phase-5a-rendering-architecture.md) — Write `docs/architecture/rendering.md` — the contract all subsequent steps implement against
- [5b: Design Decisions](phase-5b-design-decisions.md) — Resolve DR-1 through DR-10 **DECISION REQUIRED**

### Infrastructure

- [5c: Render Plugin Separation](phase-5c-render-plugin-separation.md) — Extract visual concerns from gameplay plugins, establish rendering/ domain
- [5d: Post-Processing Pipeline](phase-5d-post-processing-pipeline.md) — Bloom tuning, screen distortion shader, chromatic aberration, additive blending
- [5e: Particle System](phase-5e-particle-system.md) — Evaluate crates, integrate, implement 6 particle types
- [5f: Temperature Palette & Data-Driven Enums](phase-5f-temperature-and-enums.md) — Temperature resource, visual composition enums, RON integration

### Entity Visuals

- [5g: Bolt Visuals](phase-5g-bolt-visuals.md) — Bolt shader, wake/trail, state communication, extra bolt distinction
- [5h: Breaker Visuals](phase-5h-breaker-visuals.md) — Per-archetype shapes, colors, auras, dash trails, states
- [5i: Cell Visuals](phase-5i-cell-visuals.md) — Per-type shapes/colors, damage states, destruction effects, special cell VFX
- [5j: Walls & Background](phase-5j-walls-and-background.md) — Wall meshes, impact flash, shield barrier, background grid

### Effects & Feedback

- [5k: Screen Effects & Feedback](phase-5k-screen-effects.md) — Screen shake, flash, desaturation, slow-mo, vignette
- [5l: Bump Grade & Failure State VFX](phase-5l-bump-and-failure-vfx.md) — Bump feedback, bolt lost, shield absorption, run end states
- [5m: Combat Effect VFX](phase-5m-combat-effect-vfx.md) — Shockwave, chain lightning, piercing beam, pulse, explode, gravity well, and more
- [5n: Visual Modifier System](phase-5n-visual-modifiers.md) — Chip effect appearance changes on bolt/breaker, diminishing returns
- [5o: Highlight Moments](phase-5o-highlight-moments.md) — Glitch text shader, per-highlight visual treatments

### UI & Screens

- [5p: Transitions](phase-5p-transitions.md) — Glitch, collapse/rebuild, random selection, upgrade existing
- [5q: HUD & Gameplay UI](phase-5q-hud-ui.md) — Diegetic HUD: timer in wall glow, lives as orbs, node progress in frame
- [5r: Chip Cards](phase-5r-chip-cards.md) — Card shape, rarity treatments, abstract symbol icons, timer pressure
- [5s: Screens](phase-5s-screens.md) — Main menu, run-end (hybrid: victory=splash, defeat=hologram), breaker select, pause, loading

### Evolution VFX

- [5t: Evolution VFX Batch 1 — Beams](phase-5t-evo-beams.md) — Nova Lance (Railgun dropped)
- [5u: Evolution VFX Batch 2 — AoE](phase-5u-evo-aoe.md) — Supernova, Gravity Well, Dead Man's Hand
- [5v: Evolution VFX Batch 3 — Chain/Spawn](phase-5v-evo-chain-spawn.md) — Chain Reaction, Split Decision, Feedback Loop, Entropy Engine
- [5w: Evolution VFX Batch 4 — Entity Effects](phase-5w-evo-entity-effects.md) — Phantom Breaker, Voltchain, ArcWelder, FlashStep, Second Wind

## Build Order Rationale

Infrastructure first because everything else depends on the rendering domain, post-processing pipeline, particle system, and visual composition enums. Entity visuals before effects because entities are the canvas that effects paint on. Screen effects before combat VFX because combat VFX use screen shake/flash/distortion. UI last because it benefits from having the full visual language established. Evolution VFX last because they're the most complex individual effects and benefit from a mature rendering pipeline.

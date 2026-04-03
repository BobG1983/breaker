# Phase 5: Visual Identity

**Goal**: Establish the stylized, shader-driven aesthetic. Not polish — identity. Every entity, effect, and screen should communicate through light, geometry, and motion — no floating text, no damage numbers, no UI overlays where visual feedback serves better.

## Architecture: rantzsoft_vfx Crate + Dispersed Game Domains

**There is no `rendering/` or `graphics/` game domain.** All rendering primitives, shaders, recipes, modifiers, and entity visual attachment live in `rantzsoft_vfx` — a game-agnostic VFX crate at workspace root. Game-specific visual concerns are dispersed to the domains that own the relevant game state.

The full architecture is documented in `docs/architecture/rendering/` (written as part of step 5a). Key documents:

| Document | What it covers |
|----------|---------------|
| [rendering/index.md](../../architecture/rendering/index.md) | Top-level overview, links to all sub-documents |
| [rendering/composition.md](../../architecture/rendering/composition.md) | Two composition paths: recipe (RON) vs direct primitive (code) |
| [rendering/rantzsoft_vfx.md](../../architecture/rendering/rantzsoft_vfx.md) | Crate scope, VfxConfig resource, camera-targeting API |
| [rendering/recipes.md](../../architecture/rendering/recipes.md) | Recipe system: phases, primitives, ExecuteRecipe, hot-reload |
| [rendering/modifiers.md](../../architecture/rendering/modifiers.md) | SetModifier/AddModifier/RemoveModifier, diminishing returns |
| [rendering/communication.md](../../architecture/rendering/communication.md) | All message types, system ordering, domain restructuring |

### Communication Pattern

| Direction | Mechanism | Examples |
|-----------|-----------|---------|
| Gameplay → VFX (entity identity) | `AttachVisuals { entity, config }` message | shape, color, glow, aura, trail |
| Gameplay → VFX (dynamic state) | `SetModifier` messages (per-frame overwrites) | speed → trail length, piercing → spikes |
| Gameplay → VFX (chip effects) | `AddModifier` / `RemoveModifier` messages | stacking with diminishing returns |
| Gameplay → VFX (event VFX) | `ExecuteRecipe` message or typed per-primitive messages | cell death recipe, bolt lost recipe |
| VFX → Gameplay (completion) | `TransitionComplete` (owned by screen/, not crate) | transition animation done |

No `*RenderState` bridge components. No `VfxKind` dispatch enum. No per-effect rendering modules.

### Domain Ownership

| Concern | Owner |
|---------|-------|
| All rendering primitives, shaders, recipes, modifiers | `rantzsoft_vfx` crate |
| VfxConfig resource (type definition) | `rantzsoft_vfx` crate |
| VfxConfig values (insert + mutation) | Game (`shared/`, debug menu, settings) |
| Temperature palette + danger vignette | `run/` domain |
| Transitions, PlayingState substates | `screen/transition/` |
| Per-screen UI (chip select, menus, pause, HUD) | `screen/<screen_name>/` |
| Diegetic HUD (timer wall, life orbs, node progress) | `screen/playing/hud/` |
| GraphicsConfig resource | `shared/` |

### Eliminated Domains

| Domain | Absorbed Into |
|--------|--------------|
| `ui/` | Per-screen UI → `screen/<screen_name>/`. Diegetic HUD → `screen/playing/hud/`. |
| `fx/` | Transitions → `screen/transition/`. Fade-out, punch scale → `rantzsoft_vfx`. |

## Prerequisites

**Bolt definitions** must be implemented before bolt visual work (step 5g). See `.claude/specs/bolt-definitions-code.md` and `docs/architecture/rendering/bolt-graphics-migration.md`.

## Design Decisions

All design decisions (DR-1 through DR-10) have been resolved. See `docs/design/graphics/decisions-required.md`.

## Audio

Phase 5 is purely visual. All audio work (SFX, music, heartbeat timer, layered intensity) belongs to Phase 6. VFX systems in Phase 5 do not emit audio events or include audio stubs.

## Subphases

Steps are ordered by dependency. Architecture and decisions (5a-5b) come first. Infrastructure steps (5c-5f, 5p) establish foundations — note that 5p (PlayingState expansion + transition styles) is infrastructure because all subsequent steps need the final state machine. Entity visuals (5g-5j) build the core look. Effects and feedback (5k-5o) add juice. UI and screens (5q-5s) complete the experience. Evolution VFX (5t-5w) are the crown jewels.

Steps 5d and 5e are independent and can be done in either order. 5p can run in parallel with 5d-5f.

### Architecture & Decisions

- [5a: Rendering Architecture](phase-5a-rendering-architecture.md) — Write `docs/architecture/rendering/` — **COMPLETE**
- [5b: Design Decisions](phase-5b-design-decisions.md) — Resolve DR-1 through DR-10 — **COMPLETE**

### Infrastructure

- [5c: Crate Setup + Plugin Separation](phase-5c-render-plugin-separation.md) — Create `rantzsoft_vfx` crate, extract visual concerns from gameplay plugins, eliminate ui/ and fx/ domains
- [5d: Post-Processing Pipeline](phase-5d-post-processing-pipeline.md) — Bloom tuning, FullscreenMaterial effects, screen distortion, chromatic aberration, additive blending via specialize()
- [5e: Particle System](phase-5e-particle-system.md) — CPU particle system in rantzsoft_vfx, emitter modes, per-primitive mapping
- [5f: Temperature Palette & Data-Driven Enums](phase-5f-temperature-and-enums.md) — RunTemperature resource, Hue/Shape/Aura/Trail enums, RON integration
- [5p: Transitions & PlayingState](phase-5p-transitions.md) — PlayingState substate expansion, 4 transition styles (Flash, Sweep, Glitch, Collapse/Rebuild)

### Entity Visuals

- [5g: Bolt Visuals](phase-5g-bolt-visuals.md) — SDF entity_glow, wake/trail via AttachVisuals, modifier messages for dynamic state
- [5h: Breaker Visuals](phase-5h-breaker-visuals.md) — Per-archetype shapes, colors, auras, trails via AttachVisuals
- [5i: Cell Visuals](phase-5i-cell-visuals.md) — Per-type shapes/colors, damage recipes, destruction recipes (dissolve/split/fracture)
- [5j: Walls & Background](phase-5j-walls-and-background.md) — Wall SDF entities, impact flash, background grid shader, shield barrier energy field

### Effects & Feedback

- [5k: Screen Effects & Feedback](phase-5k-screen-effects.md) — Screen shake, flash, desaturation, slow-mo, vignette, distortion buffer
- [5l: Bump Grade & Failure State VFX](phase-5l-bump-and-failure-vfx.md) — Bump recipes, bolt lost recipes, life lost VFX, run end states
- [5m: Combat Effect VFX](phase-5m-combat-effect-vfx.md) — Recipe authoring for all chip effects
- [5n: Visual Modifier System](phase-5n-visual-modifiers.md) — ModifierStack, DR curves, SetModifier/AddModifier handlers
- [5o: Highlight Moments](phase-5o-highlight-moments.md) — GlitchText overlay (Text2d + child GlitchMaterial), per-highlight recipes

### UI & Screens

- [5q: HUD & Gameplay UI](phase-5q-hud-ui.md) — Timer wall gauge shader, life orbs, node progress ticks (count to next boss)
- [5r: Chip Cards](phase-5r-chip-cards.md) — Entity composition, rarity treatments, holographic shader, abstract symbol icons
- [5s: Screens](phase-5s-screens.md) — Main menu, run-end (hybrid: victory=splash, defeat=hologram), breaker select, pause, loading

### Evolution VFX

- [5t: Evolution VFX Batch 1 — Beams](phase-5t-evo-beams.md) — Nova Lance (Railgun dropped)
- [5u: Evolution VFX Batch 2 — AoE](phase-5u-evo-aoe.md) — Supernova, Gravity Well, Dead Man's Hand
- [5v: Evolution VFX Batch 3 — Chain/Spawn](phase-5v-evo-chain-spawn.md) — Chain Reaction, Split Decision, Feedback Loop, Entropy Engine
- [5w: Evolution VFX Batch 4 — Entity Effects](phase-5w-evo-entity-effects.md) — Phantom Breaker, Voltchain, ArcWelder, FlashStep, Second Wind

## Build Order Rationale

Infrastructure first because everything else depends on the VFX crate, post-processing pipeline, particle system, and visual composition enums. Entity visuals before effects because entities are the canvas that effects paint on. Screen effects before combat VFX because combat VFX use screen shake/flash/distortion. UI last because it benefits from having the full visual language established. Evolution VFX last because they're the most complex individual effects and benefit from a mature rendering pipeline.

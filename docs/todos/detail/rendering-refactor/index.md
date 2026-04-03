# Rendering Architecture

There is **no `rendering/` or `graphics/` game domain**. All rendering primitives, shaders, recipes, modifiers, and entity visual attachment live in `rantzsoft_vfx` — a game-agnostic VFX crate. Game-specific visual concerns are dispersed to the domains that own the relevant game state (see [Communication](communication.md) for the full domain map).

**Core principle: visuals are composed from primitives, not coded per-effect.** The crate knows how to draw rings, beams, particles, glows, and distortions. A "shockwave" is a recipe: "expanding ring + radial distortion + small screen shake." The recipe is a named RON asset.

**Core principle: dynamic visuals are driven by modifiers, not render state components.** Gameplay domains don't maintain `*RenderState` bridge components. Instead, they send `SetModifier` / `AddModifier` / `RemoveModifier` messages to the VFX crate, which maintains the visual state per entity.

## Contents

| Document | Contents |
|----------|----------|
| **Build Specs** | |
| [Module Map](module-map.md) | rantzsoft_vfx crate module tree, lib.rs exports, prelude, naming conventions |
| [System Scheduling](scheduling.md) | VfxSet enum, internal ordering, game integration, system-to-set mapping |
| [Material Layouts](materials.md) | Full AsBindGroup structs, WGSL bindings, std140 layouts for all materials |
| [Error Handling](error-handling.md) | Edge case catalog, warn-and-skip policy, per-situation behavior table |
| [Screen Migration](screen-migration.md) | ui/ and fx/ → screen/ migration table, target module tree, plugin changes |
| **Design** | |
| [Composition Model](composition.md) | Two composition paths (recipe vs direct primitive), what goes where |
| [rantzsoft_vfx Crate](rantzsoft_vfx.md) | Crate scope, what lives in the crate vs game, VfxConfig, camera-targeting API |
| [Types](types.md) | Hue, Shape, Aura, Trail, GlowParams, typed visual parameters, VisualModifier |
| [Recipes](recipes.md) | Recipe system: phases, primitives, ExecuteRecipe, dispatch algorithm, hot-reload, RON format |
| [Modifiers](modifiers.md) | Set/Add/Remove modifier messages, diminishing returns, ModifierConfig |
| [Entity Visuals](entity_visuals.md) | AttachVisuals, EntityVisualConfig, RON rendering substructs |
| [Shaders](shaders.md) | Concept shader catalog: entity_glow, aura, trail, primitives, post-processing, special |
| [Screen Effects](screen_effects.md) | Post-processing pipeline, FullscreenMaterial, shake, flash, distortion, vignette, CRT |
| [Walls & Background](walls_and_background.md) | Wall rendering, background grid shader, shield barrier lifecycle |
| [Slow Motion](slow_motion.md) | Time\<Virtual\> dilation, smooth ramp, gotchas |
| [Temperature](temperature.md) | RunTemperature resource, instant snap on transition, palette endpoints |
| [Transitions](transitions.md) | 4 styles (Flash, Sweep, Glitch, Collapse/Rebuild), PlayingState substates |
| [HUD](hud.md) | Diegetic HUD: timer wall gauge, life orbs, node progress ticks |
| [Particles](particles.md) | CPU particle system, soft cap, emitter modes, per-primitive mapping |
| [Chip Cards](chip_cards.md) | Card entity composition, rarity treatments, holographic shader, timer pressure |
| [Screens](screens.md) | Screen rendering principles: main menu, run-end, pause, loading, breaker select |
| [Headless Mode](headless.md) | How rantzsoft_vfx operates without rendering (scenario runner) |
| [Communication](communication.md) | All message types, gameplay↔VFX data flow, system ordering, domain restructuring |
| [Bolt Migration](bolt-graphics-migration.md) | Delta from data-driven bolts to new graphics system |
| [Placeholder Defaults](placeholder-defaults.md) | Phase 5 placeholder tuning values (tuned in Phase 7) |

## Design Reference

- [design/graphics/](../../design/graphics/index.md) — visual identity, color palette, gameplay elements
- [design/graphics/data-driven-graphics.md](../../design/graphics/data-driven-graphics.md) — RON-driven visual composition philosophy
- [design/graphics/effects-particles.md](../../design/graphics/effects-particles.md) — effect VFX and particle design direction
- [design/graphics/catalog/](../../design/graphics/catalog/index.md) — every visual element with status and priority

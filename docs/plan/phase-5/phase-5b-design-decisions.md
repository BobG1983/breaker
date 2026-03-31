# 5b: Design Decisions — COMPLETE

**Goal**: Resolve all open design decisions (DR-1 through DR-10) from `docs/design/graphics/decisions-required.md` before implementation begins.

## All Decisions Resolved

| Decision | Resolution | Affects |
|----------|-----------|---------|
| **DR-1**: HUD Style | Diegetic/Integrated — timer wall gauge, life orbs, node progress ticks | 5q |
| **DR-2**: Run-End Screen | Hybrid — victory = splash (celebratory), defeat = hologram (contemplative) | 5s |
| **DR-3**: Shield Color | Patterned white — hexagonal/honeycomb pattern, distinguished by pattern not color | 5j, 5l |
| **DR-4**: Highlight Treatments | Contextual emphasis — glitch text label + per-highlight game element VFX | 5o |
| **DR-5**: Chip Card Icons | Abstract geometric symbols per effect type | 5r |
| **DR-6**: Grid Density | Configurable via debug menu, stored in VfxConfig | 5j |
| **DR-7**: CRT/Scanline | Off by default, configurable in debug menu, stored in VfxConfig | 5d |
| **DR-8**: Transitions | 4 + extensible — Flash, Sweep, Glitch, Collapse/Rebuild | 5p |
| **DR-9**: Evolution VFX | All reviewed against behaviors — corrections applied | 5t-5w |
| **DR-10**: Discovery UI | Visual language defined, screen deferred to Phase 10 | 5s |

## Architecture Decisions

| Decision | Resolution | Affects |
|----------|-----------|---------|
| Entity visuals | `AttachVisuals { entity, config }` message with `EntityVisualConfig` struct | 5f-5j |
| VFX composition | Compositional primitives + RON recipes via `ExecuteRecipe`, no per-effect modules | 5m, 5t-5w |
| Dynamic visuals | `SetModifier`/`AddModifier`/`RemoveModifier` messages, no `*RenderState` components | 5n |
| Particle system | CPU particles in `rantzsoft_vfx`, soft cap 8192, no separate crate | 5e |
| Rendering config | `VfxConfig` in crate (read-only), `GraphicsConfig` in shared/ | 5d |
| Aura materials | Single `AuraMaterial` with variant uniform, not separate types | 5h |

## Output

All resolutions documented in `docs/design/graphics/decisions-required.md` and reflected in `docs/architecture/rendering/`.

# 5b: Design Decisions — RESOLVED

**Goal**: Resolve all open design decisions (DR-1 through DR-10) from `docs/design/graphics/decisions-required.md` before implementation begins.

## All Decisions Resolved

| Decision | Resolution | Affects |
|----------|-----------|---------|
| **DR-1**: HUD Style | Diegetic/Integrated — timer in wall glow, lives near breaker, progress in playfield frame | 5q |
| **DR-2**: Run-End Screen | Hybrid — victory = splash (celebratory), defeat = hologram (contemplative) | 5s |
| **DR-3**: Shield Color | Patterned white — hexagonal/honeycomb pattern, distinguished by pattern not color | 5j, 5l |
| **DR-4**: Highlight Treatments | Contextual emphasis — glitch text label + per-highlight game element VFX | 5o |
| **DR-5**: Chip Card Icons | Abstract geometric symbols per effect type | 5r |
| **DR-6**: Grid Density | Configurable via debug menu, stored in RenderingDefaults RON | 5j |
| **DR-7**: CRT/Scanline | Off by default, configurable in debug menu, stored in RenderingDefaults RON | 5d |
| **DR-8**: Transitions | 4 + extensible — enum + module pattern (rendering/transition/<name>/*) | 5p |
| **DR-9**: Evolution VFX | All reviewed against behaviors — corrections applied | 5t-5w |
| **DR-10**: Discovery UI | Visual language defined (locked/unknown/almost-unlocked states), screen deferred to Phase 10 | 5s |

## Architecture Decisions

| Decision | Resolution | Affects |
|----------|-----------|---------|
| Visual identity components | Separate generic components (Shape, Color, AuraType, etc.) — not bundled per entity type | 5f |
| Render messages | Module-owned Bevy messages (not observers), VfxKind enum for RON dispatch only | 5a, 5c |
| Rendering config | New RenderingDefaults RON + RenderingConfig resource via rantzsoft_defaults pipeline | 5d |
| Particle system | Custom `rantzsoft_particles` crate — no external dependency | 5e |

## Output

All resolutions documented in `docs/design/graphics/decisions-required.md`. No code changes needed — this was a planning step.

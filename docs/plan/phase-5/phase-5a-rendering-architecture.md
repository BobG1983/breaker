# 5a: Rendering Architecture Document — COMPLETE

**Goal**: Write `docs/architecture/rendering/` — the complete architectural specification for the VFX system — before any code is written. This document set is the contract that all subsequent Phase 5 steps implement against.

## Output

The architecture is documented across 20 files in `docs/architecture/rendering/`. See `docs/architecture/rendering/index.md` for the full index.

Key documents:
- `composition.md` — Two composition paths (recipe vs direct primitive)
- `rantzsoft_vfx.md` — Crate scope, VfxConfig, VfxLayer trait, trail cleanup, recipe timing
- `types.md` — Hue, Shape, Aura, Trail, GlowParams, VisualModifier, ModifierKind
- `recipes.md` — Recipe system: phases, primitives, ExecuteRecipe, hot-reload, RON format
- `modifiers.md` — SetModifier/AddModifier/RemoveModifier, diminishing returns
- `entity_visuals.md` — AttachVisuals, EntityVisualConfig, RON rendering substructs
- `shaders.md` — Full shader catalog: entity_glow, aura, trail, primitives, post-processing, special
- `communication.md` — All message types, system ordering, domain restructuring

## Dependencies

- None — this was the first Phase 5 step

## Verification

- Architecture docs are internally consistent
- All patterns needed by later steps are documented
- Research docs in `docs/architecture/rendering/research/` back key Bevy 0.18 API decisions

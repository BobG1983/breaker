# 5o: Highlight Moments

**Goal**: Replace text-based highlight popups with contextual emphasis — each highlight gets a GlitchText label (Text2d + child overlay) plus per-highlight game element VFX.

Architecture: `docs/architecture/rendering/shaders.md` — glitch_text.wgsl section

## What to Build

### 1. GlitchText Primitive

Crate-owned `SpawnGlitchText { position, text, size, color, duration }` message + `GlitchText` PrimitiveStep for recipes.

Implementation: Bevy `Text2d` entity with monospace font renders the text. A child overlay entity (quad mesh with `GlitchMaterial`) applies glitch effects as a fragment shader:
- Scanlines: horizontal bands modulating alpha
- Chromatic split: RGB channel UV offset
- Block jitter: hash-based block displacement
- Punch scale: Transform animation on parent (scale 1.0 → 1.3 → 1.0 over ~0.15s)

Auto-despawns after `duration` seconds.

### 2. Per-Highlight Recipes

Each highlight type = `ExecuteRecipe` with a highlight recipe. Recipe contains `GlitchText` step + per-highlight game element VFX:

| Highlight | Text | Game Element VFX Steps |
|-----------|------|----------------------|
| Close Save | "SAVE." | ScreenFlash (barrier color) |
| Mass Destruction | "OBLITERATE." | ExpandingRing (center) |
| Combo King | "COMBO." | SparkBurst (near bolt) |
| Pinball Wizard | "RICOCHET." | SparkBurst (at wall) |
| First Evolution | "EVOLVE." | ScreenFlash + ChromaticAberration |
| Nail Biter | "CLUTCH." | VignettePulse |
| Perfect Streak | "STREAK." | SparkBurst (near breaker) |
| Fast Clear | "BLITZ." | ScreenFlash |
| No Damage Node | "FLAWLESS." | ExpandingRing |
| Speed Demon | "DEMON." | SparkBurst (along bolt trail) |
| Untouchable | "GHOST." | GlowMotes (around breaker) |
| Comeback | "SURGE." | ScreenFlash (green tint) |
| Perfect Node | "PERFECT." | ScreenFlash + ScreenShake(Medium) |
| Most Powerful Evolution | "APEX." | ScreenFlash + ChromaticAberration + ScreenShake(Heavy) |

### 3. Highlight Dispatch

On `HighlightTriggered { kind }`: `run/` domain sends `ExecuteRecipe` with the highlight's recipe, position at the relevant game element.

Node-end highlights (FLAWLESS, BLITZ, PERFECT, GHOST, STREAK, DEMON) display on the chip select screen rather than during gameplay.

### 4. Remove Existing Text Popups

Remove current highlight system (floating text with PunchScale + FadeOut). Replace with GlitchText recipes.

## Dependencies

- **Requires**: 5c (crate), 5d (bloom for text glow), 5k (screen effects for shake/flash)
- **Enhanced by**: 5g-5j (entity visuals that highlight VFX interact with)

## Verification

- All highlights produce GlitchText labels at correct positions
- Glitch shader shows scanlines, chromatic split, block jitter
- Labels punch-scale on appear and auto-despawn on duration
- No plain floating text remains
- Labels don't distract from gameplay (brief, positioned near relevant element)
- All existing tests pass

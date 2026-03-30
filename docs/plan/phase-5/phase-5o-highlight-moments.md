# 5o: Highlight Moments

**Goal**: Replace text-based highlight popups with contextual emphasis — each highlight gets a stylized glitch text label plus per-highlight game element VFX at the relevant screen location.

## What to Build

### 1. Glitch Text Shader

Custom text rendering shader for highlight labels:
- Scan line overlay (subtle horizontal lines)
- Chromatic split (slight RGB offset on text edges)
- Jitter (micro-movement, text is never perfectly still)
- Uses Display typography (Orbitron-Bold) at large size
- Punch-scale animation (scale up briefly, settle)
- Scan-line dissolve fade out (~0.3s)

This shader is reusable — also used by other UI text (title screen, evolution names).

### 2. Per-Highlight Visual Treatments (Contextual Emphasis)

Each highlight type gets a stylized glitch text label PLUS a game element VFX at the relevant location:

| Highlight | Text Label | Location | Game Element VFX |
|-----------|-----------|----------|-----------------|
| Close Save | "SAVE." | Bottom edge near barrier | Barrier flashes |
| Mass Destruction | "OBLITERATE." | Center-screen over cell field | Cell field pulses |
| Combo King | "COMBO." | Near bolt's current position | Bolt trail intensifies |
| Pinball Wizard | "RICOCHET." | At the wall the bolt bounced from | Wall streak effect |
| First Evolution | "EVOLVE." | Center-screen with evolution-tier glow | Screen glow shift |
| Nail Biter | "CLUTCH." | Near the timer | Timer pulses |
| Perfect Streak | Label TBD | Near breaker | Breaker glow intensifies |
| Fast Clear | Label TBD | Center-screen | Cell field flash |
| No Damage Node | Label TBD | Center-screen | Shield shimmer |
| Speed Demon | Label TBD | Near bolt trail | Bolt trail flares |
| Untouchable | Label TBD | Near breaker | Breaker aura pulse |
| Comeback | Label TBD | Near timer | Timer flash green |
| Perfect Node | Label TBD | Center-screen | Full playfield flash |
| Most Powerful Evolution | Label TBD | Center-screen with max glow | Screen-wide glow shift |

### 3. Remove Existing Text Popups

Replace the current highlight system (floating text with PunchScale + FadeOut):
- Remove "CLOSE SAVE!", "MASS DESTRUCTION!", "COMBO KING!", etc. text entities
- Replace with glitch text labels at correct positions
- Duration: ~0.5-1.0s (must not distract from active gameplay)

### 4. Translation Layer

Existing highlight gameplay messages → module-owned render message:
- `HighlightTriggered { kind }` → `SpawnHighlightLabelVfx { kind, text, position, glow_color }`
- rendering/vfx/highlight/ owns the message, system, and shader logic

## Dependencies

- **Requires**: 5c (rendering/), 5d (post-processing for bloom on text)
- DR-4 resolved: contextual emphasis (glitch text + game element VFX per highlight)
- **Enhanced by**: 5k (screen effects — highlights may trigger micro-shake)

## Catalog Elements Addressed

From `catalog/feedback.md` (Highlight Moment Popups):
- Highlight system (shared): PLACEHOLDER → glitch text shader
- "SAVE." (Close Save): PLACEHOLDER → glitch text
- "OBLITERATE." (Mass Destruction): PLACEHOLDER → glitch text
- "COMBO." (Combo King): PLACEHOLDER → glitch text
- "RICOCHET." (Pinball Wizard): PLACEHOLDER → glitch text
- "EVOLVE." (First Evolution): PLACEHOLDER → glitch text
- "CLUTCH." (Nail Biter): PLACEHOLDER → glitch text
- Other highlights (9 more): PLACEHOLDER → glitch text

From `catalog/systems.md`:
- Glitch text shader: NONE → implemented

## Verification

- All highlights produce glitch text labels at correct positions
- Glitch shader shows scan lines, chromatic split, jitter
- Labels punch-scale on appear and dissolve on fade
- No plain floating text remains for any highlight
- Labels don't distract from gameplay (brief, positioned away from action)
- All existing tests pass

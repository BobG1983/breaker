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
| Perfect Streak | "STREAK." | Near breaker | Breaker glow intensifies |
| Fast Clear | "BLITZ." | Center-screen | Cell field flash |
| No Damage Node | "FLAWLESS." | Center-screen | Shield shimmer |
| Speed Demon | "DEMON." | Near bolt trail | Bolt trail flares |
| Untouchable | "GHOST." | Near breaker | Breaker aura pulse |
| Comeback | "SURGE." | Near timer | Timer flash green |
| Perfect Node | "PERFECT." | Center-screen | Full playfield flash |
| Most Powerful Evolution | "APEX." | Center-screen with max glow | Screen-wide glow shift |

**Node-end highlights** (FLAWLESS, BLITZ, PERFECT, GHOST, STREAK, DEMON): These are determined at node completion, not during live gameplay. They should display on the chip select screen rather than during gameplay — shown briefly as the chip select appears, before the player focuses on card choices.

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

## What This Step Builds

- Glitch text shader (scan lines + chromatic split + jitter + punch scale + scan-line dissolve)
- 14 highlight labels with contextual emphasis (glitch text + per-highlight game element VFX)
- Translation layer: HighlightTriggered → SpawnHighlightLabelVfx
- Remove all existing floating text popups
- Node-end highlights display on chip select screen

## Verification

- All highlights produce glitch text labels at correct positions
- Glitch shader shows scan lines, chromatic split, jitter
- Labels punch-scale on appear and dissolve on fade
- No plain floating text remains for any highlight
- Labels don't distract from gameplay (brief, positioned away from action)
- All existing tests pass

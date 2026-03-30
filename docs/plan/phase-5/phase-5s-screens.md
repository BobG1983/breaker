# 5s: Screens

**Goal**: Overhaul all non-gameplay screens to match the visual identity. Main menu, run-end, breaker select, pause, and loading screens.

## **DECISION REQUIRED: DR-2 (Run-End Screen)**

Run-end style must be resolved in 5b. Options:
- **Scorecard Hologram**: Calm debriefing, stats as floating holographic display
- **Victory/Defeat Splash**: Dramatic reveal, stats slam in with energy effects
- **Hybrid**: Victory gets splash, defeat gets hologram

## What to Build

### 1. Main Menu — Interactive Idle

Current: Orbitron title + menu items. No interactive idle.

Target:
- Playfield visible with grid background, walls present, no cells
- Breaker + bolt fully interactive (player can move and bump casually)
- Menu options as holographic/projected text, positioned to not interfere with play area
- Cool palette (early-run temperature), low particle density
- Relaxed "exhale" mood (Pillar 1)

### 2. Run-End Screen — DR-2

Implement the chosen style from DR-2. Both options share these elements:
- Run outcome (victory/defeat)
- Nodes cleared
- Highlight moments from the run
- Flux earned
- Notable build milestones
- Run seed (monospace Data typography, prominent, copy-to-clipboard affordance)

**Defeat presentation** is context-sensitive:
- Early death (nodes 1-3): minimal fanfare, quick summary, fast "try again"
- Late death (nodes 6+): show what was forming — "almost unlocked" evolution teases
- Spectacular death (high highlight count): highlight reel of peak moments

### 3. Run-End "Almost Unlocked" Teases

On defeat:
- Evolutions 1 chip away: show evolution name + icon (description hidden)
- Achievements close to triggering: show name + progress bar
- Turns defeat into discovery (Pillar 7)

### 4. Breaker Select Screen

Current: Text-only cards.

Target:
- Per-archetype visual preview (shape, color, aura visible)
- Color coding per archetype
- Selection animation (archetype activates on hover)

### 5. Pause Menu

Current: "PAUSED" text + options.

Target:
- Desaturation overlay on gameplay behind pause
- Glitch text treatment on "PAUSED" (from 5o shader)
- Menu options in Body typography

### 6. Loading Screen

Current: "Loading..." + cyan progress bar.

Target:
- Style-consistent loading indicator
- Background uses void color with subtle grid
- Temperature-appropriate color (cool, since it's pre-run)

## Dependencies

- **Requires**: 5c (rendering/), 5b (DR-2 resolved), 5d (post-processing: desaturation for pause), 5f (temperature palette), 5h (breaker visuals for archetype previews), 5o (glitch text shader), 5q (HUD typography established)
- **Enhanced by**: 5k (screen effects for run-end dramatic moments)

## Catalog Elements Addressed

From `catalog/ui-screens.md` (Screens):
- Loading screen: PLACEHOLDER → styled
- Main menu: PLACEHOLDER → interactive idle
- Run setup (breaker select): PLACEHOLDER → visual previews
- Chip select screen: handled in 5r (separate step)
- Pause menu: PLACEHOLDER → desaturation + glitch text
- Run-end screen: PLACEHOLDER → DR-2 style
- Run-end "almost unlocked": NONE → implemented

From `catalog/ui-screens.md` (Typography):
- Display font (Orbitron-Bold): PARTIAL → full glitch treatment
- Body font (Rajdhani-Medium): PARTIAL → scan line treatment

## Verification

- Main menu is interactive (bolt bounces, breaker moves)
- Run-end shows correct content with chosen style
- Defeat shows "almost unlocked" teases when applicable
- Breaker select shows archetype visual previews
- Pause overlays desaturation on gameplay
- All existing tests pass

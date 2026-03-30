# 5s: Screens

**Goal**: Overhaul all non-gameplay screens to match the visual identity. Main menu, run-end, breaker select, pause, and loading screens.

## Run-End Style: Hybrid (Context-Sensitive)

- **Victory**: Splash treatment — stats slam in with energy effects, screen shake per reveal, celebratory
- **Defeat**: Hologram treatment — floating holographic display, stats appear one by one, calm/contemplative, includes "almost unlocked" teases

## What to Build

### 1. Main Menu — Interactive Idle

Current: Orbitron title + menu items. No interactive idle.

Target:
- Playfield visible with grid background, walls present, no cells
- Breaker + bolt fully interactive (player can move and bump casually)
- Menu options as holographic/projected text, positioned to not interfere with play area
- Cool palette (early-run temperature), low particle density
- Relaxed "exhale" mood (Pillar 1)

### 2. Run-End Screen (Hybrid)

Victory gets the splash treatment, defeat gets the hologram treatment. Both share these elements:
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

- **Requires**: 5c (rendering/), 5d (post-processing: desaturation for pause), 5f (temperature palette), 5h (breaker visuals for archetype previews), 5o (glitch text shader), 5q (HUD typography established)
- **Enhanced by**: 5k (screen effects for run-end dramatic moments)
- DR-2 resolved: hybrid (victory=splash, defeat=hologram)

## What This Step Builds

- Main menu interactive idle (playfield + breaker + bolt, no cells, cool palette)
- Run-end screen: victory = splash (stats slam in), defeat = hologram (stats appear calmly)
- Context-sensitive defeat presentation (early/late/spectacular)
- "Almost unlocked" teases on defeat (evolution name + icon)
- Breaker select screen with archetype visual previews
- Pause menu with desaturation overlay + glitch text
- Loading screen with styled indicator + void + grid
- Full glitch treatment on Display font (Orbitron-Bold)
- Scan line treatment on Body font (Rajdhani-Medium)

## Verification

- Main menu is interactive (bolt bounces, breaker moves)
- Run-end shows correct content with chosen style
- Defeat shows "almost unlocked" teases when applicable
- Breaker select shows archetype visual previews
- Pause overlays desaturation on gameplay
- All existing tests pass

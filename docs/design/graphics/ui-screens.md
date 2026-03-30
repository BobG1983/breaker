# UI & Screens

How menus, HUD elements, and non-gameplay screens should look. All UI follows the abstract neon identity — no realistic materials, no paper textures, no drop shadows.

## Typography

### Style: Glitched/Stylized Projection

All text in the game looks like it's being **projected by a digital system** — not printed, not hand-drawn, not rendered on a surface. Text is light projected into the void.

- **Scan lines**: Very subtle horizontal line overlay on text (especially titles and headers)
- **Chromatic split**: Slight RGB offset on text edges (configurable intensity via debug menu)
- **Jitter**: Micro-movement on non-interactive text — text is never perfectly still. The projection is alive.
- **Font family**: Clean geometric sans-serif base (Orbitron, Rajdhani, or Exo 2 weight range) — readable, futuristic, not retro-terminal. The glitch effects are overlays on a legible base font, not a "glitch font" that sacrifices readability.

### Hierarchy

| Level | Usage | Treatment |
|-------|-------|-----------|
| Display | Title screen, run-end headers, evolution names | Large, full glitch treatment (scan lines + chromatic + jitter), bold weight |
| Heading | Screen titles, chip card names, highlight labels | Medium, moderate glitch (scan lines + slight chromatic), medium weight |
| Body | Chip descriptions, stat text, settings labels | Small, minimal glitch (scan lines only), regular weight. Readability first. |
| Data | Timer, score, combo counter (if any) | Monospace variant, clean/stable (no jitter — data must be instantly readable) |

## Chip Select Screen

The chip select is a key decision moment with a countdown timer (Pillar 5). Cards must be readable fast, feel premium, and communicate rarity instantly.

### Card Layout

Each chip card contains:
- **Name**: Heading-level typography at the top
- **Icon/illustration**: Central visual representing the chip's effect (abstract, matching the game's visual language)
- **Description**: Body-level text explaining the effect
- **Rarity border**: Glowing outline color matching the rarity tier (see `color-palette.md`)

### Card Shape

Cards have a **"cyber chip" outline** — not a standard rectangle. Think: circuit board edge, with angular notches or connection points along the border. The outline is a glowing line, not a solid fill. The card interior is semi-transparent (the void shows through slightly).

### Rarity Treatments

| Rarity | Border | Background | Special Effect |
|--------|--------|------------|----------------|
| Common | White/silver glow line | Near-transparent | None |
| Rare | Electric blue glow line | Faint blue tint | Subtle pulse animation |
| Epic | Magenta glow line | Faint magenta tint | Shimmer animation (brightness wave travels across card) |
| Legendary | Gold glow line, thicker | Warm amber tint | Particle aura around card edges, animated energy |
| Evolution | Prismatic/holographic shifting border | Holographic background shader (Balatro polychrome reference) | Full holographic treatment — color shifts with viewing angle / cursor position, rainbow reflections |

### Selection State

Cards respond to being the "current selection" (the card under the cursor/selection indicator, before confirmation):
- **Unselected**: Base rarity treatment, slightly recessed (dimmer, smaller)
- **Selected (hovering)**: Card scales up slightly, border brightens, rarity animation intensifies. A unique selection animation plays — card "activates" with a brief energy pulse from center outward.
- **Confirmed**: Card flashes bright, then the effect is dispatched. Card collapses/absorbs into the player's build.

### Timer Pressure Visualization

The countdown timer creates urgency beyond just a ticking number:
- **Audio pulse**: Background heartbeat/pulse sound that accelerates as time drops. This is the primary pressure signal.
- **Timer visual**: The timer itself is a visible element (bar, circle, or number) that uses the Data typography style — clean, stable, immediately readable.
- **Critical time**: When timer enters the last 20-30%, the pulse accelerates noticeably. The timer visual may shift to danger color (red-orange).

## Main Menu

### Interactive Idle

The main menu IS a gameplay scene — the breaker is on screen and the player can bounce a bolt around casually. No score, no timer, no objectives. Just a toy to play with while considering menu options.

- **Playfield**: Standard playfield with grid background, but no cells. The walls are present.
- **Breaker + bolt**: Fully interactive. The player can move the breaker and bump the bolt.
- **Menu options**: Overlaid as holographic/projected text elements, positioned to not interfere with the play area (side panel or top).
- **Mood**: Relaxed. The palette is cool (early-run temperatures). Particle density is low. The idle scene is the "exhale" (Pillar 1).

## Run-End Screen

The run-end shows highlights from the run ("Every Run Tells a Story" — Pillar 9). The exact style is **undecided** between two candidates — see `decisions-required.md`. Both options are documented here:

### Option A: Scorecard Hologram
- Floating holographic display of run stats and highlights
- Dark background, the scorecard is a projected light construct
- Stats appear one at a time with subtle animation
- Highlights are listed with their values and brief visual indicators
- Fits the sci-fi aesthetic, feels like a mission debriefing

### Option B: Victory/Defeat Splash
- Big dramatic reveal — stats slam in with energy effects
- Highlights animate with impact (screen shake, particle bursts on each reveal)
- The run-end screen itself is a spectacle, the final "moment" of the run
- More emotional, more celebratory (or mourning on defeat)

Both options should display the same information: run outcome, nodes cleared, highlight moments, flux earned, and notable build milestones.

## HUD (During Gameplay)

The HUD style is **undecided** between diegetic (integrated into the game world) and neon dashboard (overlaid holographic readouts). See `decisions-required.md`.

### Elements That Must Be Displayed
- **Node timer**: Time remaining for the current node
- **Lives**: Remaining bolt lives
- **Node progress**: Which node of the run (e.g., 3/8)
- **Active chips**: Some indication of the current build (may be optional/toggleable)

### Shared Requirements (Regardless of Style)
- Must not obscure the playfield corners (bolt paths)
- Timer must be instantly readable in peripheral vision
- Lives must be glanceable without looking away from the bolt
- All HUD text uses the Data typography style (monospace, stable, no jitter)

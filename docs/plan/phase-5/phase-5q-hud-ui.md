# 5q: HUD & Gameplay UI

**Goal**: Implement a diegetic HUD — all gameplay information is integrated into the game world. No overlaid panels or dashboards. Timer, lives, and node progress live inside the playfield.

## What to Build

### 1. Timer as Wall Glow

Current: Plain text number in status panel, proportional font.

Target:
- Timer is a glow bar along the **top wall** — wall glow intensity/length represents time remaining
- As time decreases, the glow recedes from both ends toward center (or one end to the other)
- Color shifts from cool (safe) to red-orange (danger) as time drops below 25%
- Must be readable in peripheral vision — the player feels the timer without looking at it
- Exact numeric time displayed as small monospace text integrated into the wall (Data typography, stable, no jitter)

### 2. Lives as Orbs

Current: `LivesCount` tracked but never displayed.

Target:
- Lives displayed as small energy orbs near the breaker or along the bottom edge
- Glanceable without looking away from bolt
- Visual depletion on life loss: orb dims and dissolves (connects to 5l failure VFX)
- Orb color follows breaker archetype accent

### 3. Node Progress in Playfield Frame

Current: `RunState.node_index` tracked but not displayed.

Target:
- Node progress integrated into the playfield frame (e.g., segment markers along a wall)
- Subtle — does not compete with gameplay
- Current node visually distinct from completed/upcoming nodes

### 4. Active Chips (Minimal)

Current: Empty "AUGMENTS" left panel container.

Target:
- The build is primarily communicated through entity visual state (bolt appearance, breaker aura, modifier effects from 5n)
- Optionally: small, subtle indicators along a playfield edge showing chip count per category
- Low priority — the visual modifier system (5n) IS the chip display
- Remove the empty side panel container

### 5. Side Panel Removal

With diegetic HUD, the side panels no longer serve a purpose:
- Remove the "AUGMENTS" left panel
- Remove the status right panel
- Playfield takes full width (or near-full with thin glowing wall borders from 5j)

### 6. Data Font (Monospace)

Current: No monospace font in assets.

Target:
- Add monospace font for timer numeric display, seed display, and any numeric data
- Timer and seed MUST use monospace for readability

## Dependencies

- **Requires**: 5c (rendering/), 5f (temperature palette for wall tinting), 5j (wall meshes — timer integrates into top wall)
- **Enhanced by**: 5o (glitch text shader could apply to node progress indicators)
- DR-1 resolved: Diegetic/Integrated

## Catalog Elements Addressed

From `catalog/ui-screens.md` (HUD Elements):
- Node timer: PLACEHOLDER → wall glow bar + monospace readout
- Lives display: NONE → energy orbs near breaker
- Node progress: NONE → playfield frame segments
- Active chips display: NONE → entity visual state (5n) is the display
- Side panels: PLACEHOLDER → removed (diegetic replaces them)
- HUD style: NONE → diegetic/integrated

From `catalog/ui-screens.md` (Typography):
- Data font (monospace): NONE → implemented

## Verification

- Timer is visible as wall glow bar and monospace readout
- Timer glow changes color at danger thresholds
- Lives orbs visible near breaker, deplete on loss
- Node progress visible in playfield frame
- No side panels remain
- All HUD info readable without distracting from gameplay
- All existing tests pass

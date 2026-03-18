# Phase 5: Visual Identity

**Goal**: Establish the stylized, shader-driven aesthetic. Not polish — identity.

## Shaders & Effects

- **Breaker shader**: Glow, trail effect on dash, tilt visualization, visual feedback on brake/settle, bump flash
- **Bolt shader**: Glow, motion trail, impact flash
- **Cell shaders**: Per-cell-type visual identity (Lock cells look locked, Regen cells pulse), damage states, destruction dissolve/shatter
- **Background**: Animated shader background that responds to game state (intensity increases with urgency)
- **Screen effects**: Screen shake on impacts, chromatic aberration on big hits, vignette on low timer
- **Particle system**: Cell destruction particles, perfect bump sparks, chip pickup effects, shockwave ring VFX

## Situation-Dependent Visuals (Pillar 9)

The visual system must reflect the *current build state*, not just the current action:
- **Bolt appearance changes with amps**: A piercing bolt looks different from a base bolt. A bolt with 3 speed stacks has a longer trail. Visual stacking = the build is visible.
- **Breaker appearance changes with augments**: Wide breaker is visually wider (already mechanical), but also glows differently with stacking. Dash trail color shifts with augment count.
- **Overclock activation VFX**: Each overclock has a distinct visual signature. Surge shockwave, multi-bolt split, shield shimmer. These are the "spectacle moments" that make builds feel real.
- **Intensity scaling**: Background, particle density, and screen effects escalate with the player's build power. A broken build late in the run should make the screen look noticeably more chaotic than node 1.

These visual layers serve Pillar 4 (Maximum Juice, Safeguarded Chaos) but also Pillar 9 — they make each run *look* unique, not just play unique. A stream viewer can see the difference between a piercing+speed build and a wide+shockwave build.

## UI Rendering

- Timer display, score, chip icons — shader-driven or clean minimalist
- Chip stack indicators visible during gameplay (small icons showing current build)
- Rarity glow on chip selection cards

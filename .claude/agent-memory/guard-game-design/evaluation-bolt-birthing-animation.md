---
name: Bolt Birthing Animation Evaluation
description: Evaluation of bolt birthing animation (scale-up), quit teardown chain, and tether beam interaction with birthing bolts
type: project
---

## Bolt Birthing Animation — Evaluation 2026-04-08

### Birthing animation concept: APPROVED
Strong juice for mid-gameplay bolt spawns (tether beam, prism, etc). Visual of bolt materializing from a point of light is good spatial storytelling.

### Duration (0.3s): TOO SLOW — recommend 0.12-0.15s
0.3s at 60fps = 18 frames of non-interactive bolt. During AnimateIn this creates dead air. During gameplay spawns, it creates a noticeable period where the bolt exists but can't interact.

### Linear lerp: WEAK — recommend ease-out
Linear scaling looks mechanical. Ease-out (fast start, smooth finish) would feel snappier and more alive, matching the game's neon/energy aesthetic.

### AnimateIn gates on birthing: QUESTIONABLE
`all_animate_in_complete` blocks the AnimateIn -> Playing transition until all birthing completes. But bolts can't be launched during birthing anyway (`LaunchFilter` excludes birthing bolts). The node could transition to Playing immediately and let birthing finish during serving state, eliminating dead air.

### Quit teardown chain: APPROVED
Routing quit through MenuState -> GameState -> AppState -> AppExit is correct infrastructure. TransitionType::None on quit route means no fade delay. State machine cleanup runs correctly. No feel impact.

### Tether beams on birthing bolts: APPROVED
Beam connecting to expanding bolts is BETTER juice than delayed connection. tick_tether_beam reads Position2D (valid during birthing) and BoltRadius (set by builder). Damage path is independent of bolt collision layers. Chain mode same verdict.

### Open concerns
- Mid-run quit path (from pause menu through full teardown cascade) not visible in this branch — verify separately
- If duration stays at 0.3s, the AnimateIn gate concern becomes critical (0.3s + fade time = significant dead air between nodes)

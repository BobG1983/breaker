# Feedback & Juice

How the game communicates events to the player through visual feedback. Core rule from Pillar 4: **all feedback is visual-only** — no floating text, no damage numbers, no text labels for bump grades. The game communicates through light, motion, and screen effects.

## Bump Grade Feedback

Bump quality is the core high-skill interaction (Pillar 3). Feedback must be instant, unambiguous, and rewarding for Perfect bumps.

| Grade | Breaker Response | Screen Response | Particle Response |
|-------|-----------------|-----------------|-------------------|
| Perfect | Bright gold/white flash on breaker. VFX burst from impact point. Full archetype glow momentarily intensifies. | Brief micro-shake (if screen shake enabled). Tiny HDR bloom spike at impact. | Spark burst from impact point — bright, vivid, numerous. |
| Early | Dim archetype-color flash on breaker. Subtle VFX at impact. | No screen effect. | Small spark burst — fewer, dimmer than Perfect. |
| Late | Dim archetype-color flash on breaker. Minimal VFX. | No screen effect. | Minimal sparks — barely visible. |
| Whiff (no bump) | No visual response on breaker. | No screen effect. | No particles. Silence IS the feedback. |

The gradient from Perfect → Whiff should be dramatically visible. Perfect is a visual celebration; Whiff is nothing. The difference teaches the player what they're aiming for.

## Failure States

Failure feedback uses a **dramatic pause** — a brief real-time slowdown that gives the player a moment to register what happened, then snaps back to full speed. This is not cosmetic slow-mo; the game clock (including the node timer) actually slows.

### Bolt Lost
- **Trigger**: Bolt passes below the playfield (no shield)
- **Visual**: Brief slow-mo (~0.3s at 30% speed). Bolt leaves a bright streak as it exits. Screen desaturates briefly. The void below "swallows" the bolt.
- **Recovery**: Snap back to full speed. New bolt spawns from breaker with a brief flash.
- **Shield absorption**: If shield absorbs the loss, the barrier at the bottom flashes bright, cracks appear on the barrier, and the bolt bounces back. No slow-mo — the save is instant and satisfying.

### Life Lost
- **Trigger**: Bolt lost when no extra lives remain would end run
- **Visual**: Longer slow-mo (~0.5s at 20% speed). Brief red-orange flash at screen edges (danger vignette). Life indicator visibly depletes.
- **Recovery**: Snap back. The loss stings but doesn't interrupt flow for long.

### Run Over (Defeat)
- **Trigger**: Final life lost or timer expired
- **Visual**: Extended slow-mo. Screen desaturates fully to near-monochrome. Final bolt trail fades. All particles slow and dim. Transition to run-end screen.
- **Tone**: Not punishing — contemplative. "That was a run. Here's what happened." (Pillar 8: failure must feel fair, fast, and forward-looking.)

### Run Won
- **Trigger**: Final node cleared
- **Visual**: Brief freeze-frame at the moment of the last cell destruction. Screen flash (white). Then transition to run-end screen with full spectacle.
- **Tone**: Celebratory. The build you made just worked.

## Screen Shake

Screen shake is a primary juice vector — it makes impacts feel physical in an otherwise abstract, weightless world. It fires on **punctual events only** (never continuous), so the contrast between stillness and shake makes each shake impactful.

### Shake Tiers

| Tier | Displacement | Duration | Events |
|------|-------------|----------|--------|
| Micro | 1-2px | 1-2 frames | Perfect bump, single cell break |
| Small | 3-5px | 3-4 frames | Cell chain (3+), shockwave fire, bolt-wall impact at high speed |
| Medium | 6-10px | 4-6 frames | Evolution trigger, large chain reaction (5+ cells), explosion effect |
| Heavy | 12-20px | 6-10 frames | Supernova evolution, screen-filling events, run-ending moments |

### Shake Characteristics
- **Direction**: Shake direction should roughly match the impact direction (bolt hitting right wall = horizontal shake, cell exploding = radial/random)
- **Decay**: Shake amplitude decays exponentially — sharp onset, quick falloff. Never a sustained wobble.
- **Stacking**: Multiple shake events in quick succession should combine (a chain reaction of 5 cells produces overlapping shakes that feel like rumble), but with a maximum cap to prevent nausea.
- **Anchor**: The bolt and breaker must remain trackable during shake. Shake moves the camera/viewport, not individual elements — everything moves together, preserving relative positions.

### Screen Shake Intensity Setting
Screen shake intensity is **configurable via debug menu** with a multiplier (0.0 = disabled, 1.0 = default, 2.0 = maximum). This allows tuning during development. The multiplier scales displacement and duration together.

## Screen Effects (Punctual Bursts)

Other screen-level effects that fire on specific events. Like screen shake, these are punctual — the contrast between calm and burst makes each burst hit harder.

| Event | Screen Effect |
|-------|--------------|
| Perfect bump | Micro-shake + brief bloom spike at impact point |
| Cell chain (3+) | Small shake + brief chromatic aberration pulse |
| Shockwave fire | Radial screen distortion from origin point + small shake |
| Explosion | Medium shake + central flash (HDR >2.0) + distortion ring |
| Evolution trigger | Medium shake + chromatic aberration + brief screen flash. "Something big just happened." |
| Bolt lost | Brief desaturation + slow-mo (see Failure States above). No shake — the silence is the feedback. |
| Shield break (last charge) | Brief screen flash at bottom edge + barrier shatter particles. Micro-shake. |
| Chain lightning | No shake — the electric arcs are the spectacle. Brief bloom at each target. |
| Gravity well activate | No shake — the distortion IS the effect. Subtle screen-space warp. |

All screen effects are **configurable via debug menu**: screen shake intensity, chromatic aberration intensity, CRT effect toggle, bloom intensity. This allows tuning during development and potential player settings later.

## Node Transitions

The game has multiple transition styles, and **selects randomly from the available pool** for each transition in/out. This variety prevents transitions from feeling repetitive across a multi-node run.

### Transition Pool

| Style | In (entering node) | Out (leaving node) |
|-------|--------------------|--------------------|
| Flash | Brief white/color flash, instant reveal | Brief flash, instant black |
| Sweep | Energy beam sweeps across screen, revealing/hiding | Reverse sweep |
| Glitch | Screen corrupts with static/distortion, then resolves | Screen corrupts then blacks out |
| Collapse/Rebuild | Elements build outward from center point | Elements collapse inward to center |

Each style should take ~0.3-0.5s. Transitions are fast — Pillar 1 says tension never stops, so transitions should not be rest moments.

The system randomly selects one In-style and one Out-style for each node transition. In and Out styles can be different (e.g., Glitch out, Sweep in).

## Memorable Moment Popups (In-Game Highlights)

When the player triggers a memorable moment during gameplay (Close Save, Mass Destruction, Combo King, etc.), a brief visual popup appears. These are currently text-based — they should be redesigned to match the visual-only philosophy:

- **Visual indicator**: A brief, distinctive visual effect at the relevant screen location (not center-screen text)
- **Punch scale**: The popup element scales up briefly (punch animation), then settles
- **No text labels**: The visual effect itself communicates what happened. A close save might flash the barrier. Mass destruction might pulse the entire cell field. Combo King might streak the bolt's trail.
- **Duration**: Very brief (~0.5-1.0s). Must not distract from active gameplay.

The exact visual treatment for each highlight type needs design — see `decisions-required.md`.

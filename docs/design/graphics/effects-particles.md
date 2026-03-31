# Effects & Particles

How triggered effects, AoE abilities, and particle systems should look. All effects follow the **layered** principle: geometric base shape enhanced with shader effects (glow, distortion, noise).

## Effect Design Principle: Geo + Shader

Every triggered effect has two layers:

1. **Geometric layer**: A clean, readable shape (circle, line, ring, arc) that communicates the effect's area and timing at a glance. This layer is what the player reads at 60fps during chaos.
2. **Shader layer**: Visual enhancement (bloom, distortion, noise, color shift) that makes the effect feel impactful and premium. This layer is what makes the effect satisfying. It is additive — it never obscures the geometric layer.

This separation means effects are always readable (geo layer) AND satisfying (shader layer).

## Triggered Effects

### Shockwave
- **Geo layer**: Expanding ring from the impact point. Ring thickness represents damage falloff.
- **Shader layer**: Radial screen distortion at the ring edge (refracts the background/grid). Bloom on the ring. Brief screen-space color shift inside the ring.
- **Duration**: Expands outward over ~0.3s, then fades. The expansion speed communicates the range.
- **Stacking**: Higher stacks = larger maximum radius. Visually, more stacks = thicker ring, brighter bloom, stronger distortion.

### Chain Lightning
- **Geo layer**: Branching lines connecting the source cell to nearby cells within range. Lines are jagged/angular (not smooth curves).
- **Shader layer**: Electric crackling effect along the lines — brightness fluctuates rapidly, small sparks at branch points. Brief bloom flash at each target cell.
- **Duration**: Near-instant — the chain appears, flickers for 1-2 frames, then fades. Speed communicates power.
- **Stacking**: More stacks = more chain targets = more branching lines visible.

### Piercing Beam
- **Geo layer**: Straight line extending from the bolt through pierced cells. Line shows the pierce trajectory.
- **Shader layer**: Bright core with bloom, narrowing toward the end. Brief distortion along the beam path.
- **Duration**: Appears on pierce, lingers for ~0.1s as a fading afterimage.

### Pulse
- **Geo layer**: Expanding filled circle from the breaker/bolt position, not just a ring — a circular area.
- **Shader layer**: Radial gradient from bright center to soft edge. Screen distortion at the edge. Cells inside the pulse briefly flash.
- **Duration**: Faster expansion than shockwave (~0.2s). Pulse is more immediate, shockwave is more dramatic.

### Explode
- **Geo layer**: Radial burst from a destroyed cell — multiple short lines radiating outward from center.
- **Shader layer**: Central flash (HDR >2.0), rapid particle spray, brief screen shake (if configured). Distortion ring at the edge.
- **Duration**: Fastest effect — flash, burst, fade in ~0.15s. Maximum immediacy.

### Gravity Well
- **Geo layer**: Circle showing the well's radius of influence. Faint radial lines pulling inward.
- **Shader layer**: Screen-space **distortion lens** — the area inside the well warps what's beneath it, bending light toward the center. The distortion intensifies toward the center. No dark void — the gravity well stays within the "light is the material" identity, using warping and refraction rather than absence of light.
- **Duration**: Persistent while active. The distortion is constant, with subtle animation (slow rotation of the distortion pattern).

### Tether Beam
- **Geo layer**: Visible line connecting two tethered bolts. Line has slight elasticity — stretches when bolts are far apart, slackens when close.
- **Shader layer**: Energy flowing along the beam (animated brightness traveling from one end to the other). Color matches bolt halo.
- **Duration**: Persistent while the tether is active. Snaps with a flash when the constraint breaks.

### Shield (Bolt-Loss Protection)
- **Geo layer**: Visible energy barrier along the bottom playfield edge. Solid line, brighter than the normal wall glow.
- **Shader layer**: Shimmering/rippling animation. Cracks appear when a charge is consumed. When the last charge breaks, the barrier shatters with particles.
- **Duration**: Persistent while charges remain.

### Speed Boost
- **Geo layer**: No persistent geo — this modifies the bolt's trail length and brightness.
- **Shader layer**: Speed lines / motion blur effect radiating from the bolt's wake. Bolt glow intensifies.

### Damage Boost
- **Geo layer**: No persistent geo — modifies bolt core brightness.
- **Shader layer**: Bolt core shifts toward hotter color (amber/white). Impact effects on cells become more intense.

### Attraction/Magnetism
- **Geo layer**: Faint curved arcs (2-3 thin lines, <0.3 HDR) between bolt and attraction target, bending in pull direction. Bolt's wake trail bends toward target.
- **Shader layer**: Lines flicker in and out with slight jitter — momentary glimpses of the force field. Brighten at close range. Intentionally minimal — attraction is ambient steering, not spectacle.

### Ramping Damage (Amp)
- **Geo layer**: Faint energy ring orbiting the bolt, growing brighter and spinning faster with each consecutive hit.
- **Shader layer**: Bolt halo shifts progressively warmer (base → amber → white-hot) as ramp counter climbs. At high stacks (6+), afterimage frames linger in wake. On whiff reset: heat drains visibly over ~0.3s, orbital ring shatters outward in dim sparks. The cooldown is a punishing visual moment.

### Random Effect (Flux)
- **Geo layer**: Brief multi-colored spark starburst from entity on selection.
- **Shader layer**: Rapid prismatic flash (~0.1s, 3-4 spectral color cycle) before resolving to selected effect's visual. Fast and subtle — Flux modifies other effects, isn't a spectacle itself.

### Quick Stop
- **Geo layer**: Brief compression (squash 2-3 frames) + small energy spark spray forward from breaker's leading edge.
- **Shader layer**: Trail from dash abruptly terminates at stop point. At high stacks, micro-distortion ripple emanates from breaker position (one frame of grid bending). "Momentum converted to stillness."

### Bump Force Boost
- **Geo layer**: Concentrated impact flash at contact point, radius scaled by force multiplier. At 2x+: compact radial ring expands from impact (~0.1s).
- **Shader layer**: At 3x+: HDR >2.0 bloom engulfs breaker's front edge. Distinct from Shockwave (which is large expanding ring) — force ring is compact and immediate.

### Time Penalty
- **Geo layer**: Red-orange energy line streaks from event source to timer's screen position, fading over ~0.2s.
- **Shader layer**: Timer HUD glitches briefly (chromatic split, scan line distortion, 2-3 frames). Timer flashes danger-red for ~0.3s. Single brief danger vignette pulse at screen edges. Connects cause to consequence visually.

## Evolution Effects

Each evolution has **completely unique, bespoke VFX**. Evolutions are the ultimate power fantasy reward — their visuals must feel like they come from a higher tier of visual quality. They should look fundamentally different from base chip effects — more complex, more particles, more screen presence.

| Evolution | Visual Direction |
|-----------|-----------------|
| Nova Lance | Massive beam along bolt trajectory — appears at max width, shrinks over short duration. Heavy bloom (HDR >2.0), screen distortion along path, chromatic aberration. |
| Voltchain | Enhanced chain lightning — 6 arcs with large max jumps, brighter/thicker than base. Bolt gains electric corona. Density from many cell-destroys in succession. |
| Phantom Bolt | Ghost bolt — translucent/phasing (alpha oscillation), spectral blue-violet core, afterimage trail (copies ARE the trail), flickering. Dim spark dissolve on loss. |
| Supernova | Chain cascade — base shockwave/bolt-spawn effects with Supernova visual marker (brighter ring, extra spark density). Spectacle is emergent from cascade overlap, not a single blast. |
| Dead Man's Hand | Pending mechanic rework. Provisional: dramatic shockwave from bolt-loss position + speed-up visual on remaining bolts. |
| Gravity Well | Larger/more intense distortion lens — wider radius (160), stronger warping, more glow motes, 4 active wells. |
| Second Wind | Invisible bottom wall materializes with bright flash (HDR >2.0) on save. Brief slow-mo (~0.1s). Wall visible momentarily then fades. |
| Entropy Engine | Prismatic flash per cell destroy (~0.1s, spectral cycle), then selected random effect fires. Bolt has persistent prismatic shimmer. No counter gauge. |
| Chain Reaction | Recursive shockwaves with escalating intensity per generation depth. Chromatic aberration scales with depth. Screen shake scales with total chain size. |
| Circuit Breaker | Three-node triangle charge indicator near bolt. Each perfect bump lights a node. On completion: nodes flash white-hot, collapse, circuit closes. Spawn + amplified shockwave. Screen flash + medium shake. |
| Split Decision | Fission effect: cell glow splits along axis, halves condense into bolt orbs (~0.15s). Energy filaments connect during split. Spawned bolts inherit parent visual modifiers. Prismatic birth trails. |
| ArcWelder | Crackling electric tether beams connecting ALL active bolts in sequence. Electric corona on all connected bolts. Chain forms visible electric web when many bolts active. |
| FlashStep | Breaker disintegrates into energy streak on dash-reversal during settling. Departure afterimage fades ~0.3s. Arrival radial burst + distortion. Light-streak connects departure/arrival 1-2 frames. |
| Mirror Protocol | Prismatic flash at bolt's impact point. Mirrored bolt emerges from flash with prismatic birth trail. Flash orientation (horizontal/vertical) reflects the mirror axis. |
| Anchor | Subtle ground-anchor glow beneath breaker while planted. Concentrated impact flash on planted bump. |
| Resonance Cascade | Persistent pulse aura around bolt — visible expanding rings at fixed interval. Larger bolt = larger rings. |

Evolution VFX should be designed and prototyped individually. Each is a visual set-piece. See `catalog/evolutions.md` for the complete list with status tracking.

## Particle Systems

### Philosophy: Adaptive Density

Particle density scales with the run's progression:
- **Early nodes**: Sparse particles. Each particle is individually bright and long-lived — trails, streaks, sparks. You can see individual particles.
- **Late nodes**: Dense particles. Many small particles creating sprays and clouds. Individual particles are smaller; the mass is the visual. This density is earned through gameplay — more effects firing = more particle emitters active.

The particle density ramp happens naturally as the player's build produces more triggered effects, each of which spawns its own particles. The system does not artificially increase particle count — the build does.

### Particle Types

| Type | Shape | Behavior | Used By |
|------|-------|----------|---------|
| Spark | Point/tiny streak | Burst outward, fade quickly, affected by gravity | Cell destruction, impact effects |
| Trail | Elongated streak | Follows emitter, fades with distance | Bolt wake, dash trail, beam afterimage |
| Shard | Small angular fragment | Burst outward with rotation, slower fade | Cell shatter (adaptive death) |
| Glow mote | Soft circle | Drifts slowly, long lifetime, ambient | Background energy sprites, gravity well ambient |
| Energy ring | Expanding circle | Expands and fades | Shockwave, pulse, bump feedback |
| Electric arc | Jagged line segment | Flickers rapidly, short lifetime | Chain lightning, electric effects |

### Cell Destruction — Adaptive

Cell destruction visuals scale with context (Pillar 4 — the screen exploding means you're winning):

| Context | Visual |
|---------|--------|
| Single cell break | Clean dissolve — cell fades with a brief spark burst. Satisfying but restrained. |
| Combo chain (2-4 cells in quick succession) | Shatter — cells fracture into glowing shards that scatter outward. Each break feels physical. |
| Chain reaction / shockwave kill (5+ cells) | Energy release — cells detonate into expanding light rings. Screen pulses. Particle density spikes. |
| Evolution-triggered mass destruction | Evolution-specific VFX plays on top of the above. Maximum spectacle. |

The system should detect the destruction context (single hit, combo, chain, evolution-triggered) and select the appropriate death effect tier.

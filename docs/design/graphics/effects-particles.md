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

## Evolution Effects

Each evolution has **completely unique, bespoke VFX**. Evolutions are the ultimate power fantasy reward — their visuals must feel like they come from a higher tier of visual quality. They should look fundamentally different from base chip effects — more complex, more particles, more screen presence.

| Evolution | Visual Direction |
|-----------|-----------------|
| Nova Lance | Massive beam effect — full-screen-height piercing laser with heavy bloom and distortion |
| Voltchain | Dense branching lightning web — screen fills with electric arcs between all targets |
| Phantom Breaker | Ghost bolt — translucent/phasing bolt with infinite piercing, spectral shader (flickering, afterimage trail, distinct non-white core color) |
| Supernova | Screen-filling explosion — radial burst from center, everything white-hot for a frame, then resolving |
| Dead Man's Hand | All bolts pulse simultaneously with shockwave rings — synchronized detonation feel |
| Railgun | Thin, hyper-bright beam that punches across the entire screen — instantaneous, no travel time |
| Gravity Well | See above (Gravity Well section) — the evolution version is larger and more intense |
| Second Wind | Invisible bottom wall briefly materializes with a bright flash when the bolt would be lost — salvation effect |

Evolution VFX should be designed and prototyped individually. Each is a visual set-piece.

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

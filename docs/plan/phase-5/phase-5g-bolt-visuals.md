# 5g: Bolt Visuals

**Goal**: Transform the bolt from a flat colored circle into an energy orb with glow, wake trail, and state-driven visual communication.

## What to Build

### 1. Bolt Base Shader

Replace the flat `Circle::new(1.0)` with a custom Material2d:
- Bright white/warm core (HDR >1.0 for bloom)
- Softer glow halo around the core
- Additive blending (from 5d pipeline)

### 2. Bolt Wake/Trail

Trailing energy wake showing direction and recent path:
- Trail particle emitter (using Trail particle type from 5e)
- Wake length scales with speed (read from `BoltRenderState`)
- Wake brightness scales with speed
- Fast bolt = long vivid trail, slow bolt = short dim trail

### 3. Bolt State Communication

Visual changes driven by `BoltRenderState`:

| State | Visual Change |
|-------|--------------|
| Speed | Wake length + core brightness scale with speed |
| Piercing active | Sharper angular glow, energy spikes on halo |
| Damage boosted | Core shifts amber/white, halo brightens |
| Shield on bolt | Additional aura ring around bolt (distinct from halo) |
| Size boosted | Glow scales proportionally with size |

### 4. BoltRenderState Component

Defined in bolt/ domain, synced each frame:
- `speed: f32` — current velocity magnitude
- `direction: Vec2` — normalized velocity direction
- `has_piercing: bool`
- `damage_multiplier: f32`
- `has_shield: bool`
- `lifespan_fraction: f32` — remaining lifespan as 0.0-1.0 (for lifespan indicator)

### 5. Bolt Serving (Hover) State

When bolt is being served (pre-launch):
- Pulsing orb at ~70% brightness
- No wake trail
- Halo breathes on 1.5t sine wave
- Snaps to full brightness on launch

### 6. ExtraBolt Distinction

Extra bolts (from multi-bolt effects) look subtly different:
- Same white core but halo tinted with archetype accent (~40% saturation)
- Shorter/thinner wake trail
- Dissolves into dim sparks on loss (instead of standard bolt-lost VFX)

### 7. ChainBolt + Tether Visual

Chain bolts have an energy filament to their anchor:
- Thin energy line (~0.4 HDR) connecting chain bolt to anchor bolt
- Line brightens at max stretch distance
- Simpler than evolution TetherBeam (5m)
- Flash + sparks when tether snaps

### 8. PhantomBolt Visual

Phantom bolts (from SpawnPhantom effect) have spectral appearance:
- Translucent/phasing visual — alpha oscillation
- Non-white core color (distinct from normal bolts)
- Spectral shader (flickering, afterimage trail)

### 9. Bolt Lifespan Indicator

Bolts with limited lifespan communicate remaining time:
- Below 30%: brightness/halo diminish
- Below 15%: flicker with increasing frequency, wake shortens
- At expiry: soft inward implosion of sparks (Spark particle type from 5e)

### 10. Bolt Spawn Moment

When a bolt spawns:
- Brief energy ring at spawn point (~0.1s, Energy Ring particle from 5e)
- Bolt materializes from point-source flash
- Multi-spawns overlap additively

## Dependencies

- **Requires**: 5c (rendering/ domain), 5d (post-processing for bloom/additive), 5e (particle system for trail/sparks), 5f (BoltVisualIdentity component)
- **Independent of**: 5h, 5i, 5j (other entity visuals)

## What This Step Builds

- Bolt base shader (energy orb with HDR core + halo + additive blending)
- Wake/trail particle emitter (Trail type, length scales with speed)
- State-driven visuals (speed, piercing, damage, shield, size — all reflected in appearance)
- BoltRenderState component (synced each frame by bolt/ domain)
- Serving/hover state (pulsing orb, no trail, halo breathes)
- ExtraBolt visual distinction (archetype-tinted halo, shorter trail)
- ChainBolt tether visual (energy filament to anchor bolt)
- PhantomBolt spectral visual (translucent, non-white core, afterimage trail)
- Lifespan indicator (dim/flicker below thresholds, implosion at expiry)
- Spawn moment VFX (energy ring + point-source flash)

## Verification

- Bolt renders as energy orb with glow and bloom
- Wake trail visible and scales with speed
- Each bolt state (piercing, damage, shield) is visually distinct
- ExtraBolt, ChainBolt, PhantomBolt each look different from base bolt
- Lifespan indicator visible on time-limited bolts
- All existing tests pass

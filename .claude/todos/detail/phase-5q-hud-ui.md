# 5q: HUD & Gameplay UI

**Goal**: Implement a diegetic HUD — all gameplay information integrated into the game world. No overlaid panels.

Architecture: `docs/architecture/rendering/hud.md`

## What to Build

### 1. Timer Wall Gauge

Separate overlay entity on top of the top wall with `timer_wall.wgsl` shader (gauge glow primitive in `rantzsoft_vfx`):
- `fill_level` uniform (0.0–1.0): full glow left of fill point, fading right
- `temperature` uniform: color shifts cool → red-orange at <25% time
- `pulse_speed` uniform: increases at low time for urgency
- Small monospace Text2d child for numeric readout
- System in `screen/playing/hud/timer_wall.rs`: spawns on `OnEnter(PlayingState::Active)`, updates fill_level each FixedUpdate from timer resource

### 2. Life Orbs

N energy orb entities at fixed positions below the breaker:
- Each orb gets `AttachVisuals` with breaker archetype tint
- Layout: evenly spaced at fixed y-offset from breaker spawn position
- Tracking system: orbs follow breaker x-position each FixedUpdate
- Life loss: rightmost orb plays dissolve recipe (Disintegrate + SparkBurst) and despawns
- Life gain (Second Wind): new orb materializes with birth recipe (ExpandingRing + GlowMotes inward)
- System in `screen/playing/hud/life_orbs.rs`

### 3. Node Progress Ticks

Tick marks on one side wall showing progress within current act/section:
- N ticks where N = nodes until next boss (resets each boss clear)
- Scales for infinite runs — always shows finite "progress within current section"
- Current node: bright glow. Completed: dim. Upcoming: very dim outline.
- Small entity_glow rectangles at fixed wall positions
- System in `screen/playing/hud/node_progress.rs`: reads RunState, spawns ticks on enter, updates brightness on node clear, re-spawns on section transition

### 4. Side Panel Removal

- Remove "AUGMENTS" left panel
- Remove status right panel
- Playfield takes full width with thin wall borders (from 5j)

### 5. Monospace Font

Add monospace font asset for timer readout, seed display, numeric data. Timer and seed MUST use monospace for readability.

## Dependencies

- **Requires**: 5c (crate), 5f (temperature for timer color), 5j (wall entities for timer overlay positioning)
- DR-1 resolved: Diegetic/Integrated

## Verification

- Timer visible as wall gauge with fill level and monospace readout
- Timer color shifts at danger thresholds
- Life orbs visible, track breaker, dissolve on loss, materialize on gain
- Node progress ticks visible, update on clear, reset on section transition
- No side panels remain
- All HUD readable without distracting from gameplay
- All existing tests pass

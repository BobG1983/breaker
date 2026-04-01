# Diegetic HUD

All HUD integrated into the game world. No overlaid panels. Lives in `screen/playing/hud/` (game-specific, NOT in the crate).

Side panels removed. Playfield takes full width with thin wall borders.

## Timer Wall

A **separate overlay entity** spawned on top of the top wall. Uses a dedicated `timer_wall.wgsl` shader in `rantzsoft_vfx` (a generic "gauge glow" primitive — game-agnostic).

**Shader uniforms:**
- `fill_level: f32` — 0.0 (empty) to 1.0 (full). Fragment shader: full glow left of fill point, fading glow right of it.
- `temperature: f32` — 0.0 (cool/blue-white) to 1.0 (hot/red-orange). Color shifts as time runs low (<25%).
- `pulse_speed: f32` — shimmer/pulse frequency. Increases at low time for urgency.
- `intensity: f32` — overall glow brightness.

**Game-side system** (`screen/playing/hud/timer_wall.rs`):
- Spawns the overlay entity on `OnEnter(PlayingState::Active)`, positioned over the top wall
- Each FixedUpdate: reads timer resource, computes `fill_level = time_remaining / time_total`
- Directly mutates the `TimerWallMaterial` component on the overlay entity (simpler than routing through the modifier system for a dedicated shader)
- At <25% time: ramps pulse_speed and shifts temperature toward 1.0
- Small monospace Text2d child entity for numeric readout

**The wall entity itself is NOT modified** — the timer overlay sits in front of it on a higher z-layer.

## Life Orbs

N energy orb entities spawned at **fixed positions** near the breaker. Each orb follows the breaker's x-position via a simple tracking system.

**Spawn:** On `OnEnter(PlayingState::Active)`, spawn one orb per life. Each orb gets `AttachVisuals` with:
- `shape: Circle`
- `color: [breaker archetype tint]`
- `glow: [small, bright, high bloom]`

**Layout:** Evenly spaced below the breaker at fixed y-offset. Orb positions: `breaker_x + (i - (n-1)/2) * spacing, breaker_y - offset_y`.

**Life loss:** The lost orb plays a dissolve recipe (`Disintegrate` + `SparkBurst`) and despawns. Rightmost orb removed first.

**Life gain (Second Wind):** New orb materializes with a birth recipe (`ExpandingRing` + `GlowMotes` converging inward).

**Tracking system** (`screen/playing/hud/life_orbs.rs`): Each FixedUpdate, orb positions update to track the breaker's x-position. Orbs don't move vertically — fixed y-offset from breaker spawn position.

## Node Progress

**Tick marks on one side wall** showing progress within the current act/section (nodes until next boss). Resets each boss clear. Scales to infinite runs — always shows "progress within current section."

**Design for infinite runs:** Section length could increase in later acts (e.g., 3 nodes early, 5 nodes mid, 7 nodes late). The tick count adapts per section. Like Balatro's ante display — always a finite, readable set of ticks regardless of total run length.

**Rendering:** Small entity_glow rectangles (Shape::Rectangle) at fixed positions along one side wall.
- Current node: bright glow, full intensity
- Completed nodes: dim glow
- Upcoming nodes: very dim outline

**Game-side system** (`screen/playing/hud/node_progress.rs`):
- Reads `RunState` for current node index and nodes-in-section count
- Spawns tick entities on `OnEnter(PlayingState::Active)`
- Updates modifier-driven brightness as nodes are cleared
- Despawns and re-spawns on section/boss transitions (tick count may change)

## What Lives Where

| Concern | Owner |
|---------|-------|
| timer_wall.wgsl shader (gauge glow primitive) | `rantzsoft_vfx` |
| Timer overlay spawn/update system | `screen/playing/hud/` |
| Life orb spawn/dissolve/birth systems | `screen/playing/hud/` |
| Node progress tick spawn/update systems | `screen/playing/hud/` |
| Numeric readout (Text2d) | `screen/playing/hud/` |

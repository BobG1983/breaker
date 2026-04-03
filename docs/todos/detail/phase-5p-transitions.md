# 5p: Transitions

**Goal**: Implement 4 transition styles with random selection and PlayingState substate expansion.

Architecture: `docs/architecture/rendering/transitions.md`

## What to Build

### 1. PlayingState Substate Expansion

Move `TransitionOut`, `ChipSelect`, `TransitionIn` from `GameState` to `PlayingState` substates:
```
GameState: Loading, MainMenu, RunSetup, Playing, RunEnd, MetaProgression
PlayingState: Active, Paused, TransitionOut, ChipSelect, TransitionIn
```

This means `OnEnter(PlayingState::Active)` fires on return from chip select — node spawning works naturally.

### 2. Flash Transition

- Out: bloom spike + temperature-tinted flash, then black
- In: bloom spike reveals scene
- Uses `TriggerScreenFlash` via `rantzsoft_vfx`

### 3. Sweep Transition

- Out: energy beam sweeps across screen, concealing
- In: energy beam sweeps, revealing
- Uses `SpawnBeam` + `SpawnSparkBurst` trailing the beam edge

### 4. Glitch Transition

- Out: chromatic aberration intensifies, scan line distortion, static noise → blackout
- In: resolves from corruption
- Uses `TriggerChromaticAberration` + `TriggerRadialDistortion` + `TriggerScreenFlash`

### 5. Collapse/Rebuild Transition

Tile-based screen effect via `collapse_rebuild.wgsl` FullscreenMaterial:
- Out: tiles shrink + slide toward center in radial wave (edges first, center last), slight rotation per tile
- In: tiles expand outward from center (center first, edges last)
- Uniforms: `progress` (0.0–1.0), `direction` (in/out), `tile_count`, `tile_seed`
- Purely screen-space — doesn't touch game entities

### 6. Random Selection

`TransitionStyle` enum: Flash, Sweep, Glitch, CollapseRebuild. Random selection via `GameRng` (seeded for deterministic replay). In and Out styles can differ.

### 7. Transition Flow

```
PlayingState::Active → NodeCleared → set TransitionOut
    → screen/ plays Out animation → on completion: set ChipSelect
    → player selects chip → set TransitionIn
    → screen/ plays In animation → on completion: set Active
    → OnEnter(Active) fires → node spawns
```

`TransitionComplete` message sent by `screen/` when In animation finishes.

~0.3–0.5s each. All transitions live in `screen/transition/`.

## Sequencing

**This step is infrastructure** — it runs alongside 5d-5f, before entity visuals (5g-5j). The PlayingState substate expansion must be in place before any step writes code that references transition states. Transition style implementation (the visual effects) can happen in parallel with 5d-5f since it needs the post-processing pipeline.

## Dependencies

- **Requires**: 5c (crate, screen/transition/ exists)
- **Enhanced by**: 5d (post-processing for flash/chromatic/distortion in Glitch style), 5e (particles for Sweep sparks)
- **Blocks**: 5g-5w (all subsequent steps use the final PlayingState machine)
- DR-8 resolved: 4 + extensible

## Verification

- All 4 transition styles work for both In and Out
- Random selection produces different combinations across nodes
- Transitions complete within 0.3-0.5s
- PlayingState substate expansion works (OnEnter/OnExit fire correctly)
- `TransitionComplete` fires on In animation completion
- All existing tests pass

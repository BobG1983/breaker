# Transitions

## Owner: screen/ domain (NOT rendering/)

Transitions are screen lifecycle — they belong in the `screen/` domain alongside state registration, cleanup, and state change routing. Transition visual effects use `rantzsoft_vfx` primitives. rendering/ is not involved.

## PlayingState Substate Expansion

**Key architectural change:** `TransitionOut`, `ChipSelect`, and `TransitionIn` move from top-level `GameState` variants to `PlayingState` substates. You never leave `GameState::Playing` during a run.

```rust
// Before:
GameState: Loading, MainMenu, RunSetup, Playing, TransitionOut, ChipSelect, TransitionIn, RunEnd, MetaProgression
PlayingState: Active, Paused

// After:
GameState: Loading, MainMenu, RunSetup, Playing, RunEnd, MetaProgression
PlayingState: Active, Paused, TransitionOut, ChipSelect, TransitionIn
```

**Why this is better:**
- `OnEnter(PlayingState::Active)` fires every time you return from ChipSelect — node spawning works naturally
- `OnExit(PlayingState::Active)` fires when leaving for transition — node cleanup works
- Systems with `run_if(in_state(GameState::Playing))` keep running throughout (run state, timer pause, etc.)
- No re-enter-same-state hack needed — PlayingState transitions within Playing

## Transition Flow

```
PlayingState::Active
    → NodeCleared
    → set PlayingState::TransitionOut
    → screen/ plays Out animation using rantzsoft_vfx primitives
    → on completion: set PlayingState::ChipSelect
    → player selects chip (or timer expires)
    → set PlayingState::TransitionIn
    → screen/ plays In animation
    → on completion: set PlayingState::Active
    → OnEnter(PlayingState::Active) fires → node spawns
```

## Transition Styles

Four styles, randomly selected per transition (In and Out can differ). Seed-driven for deterministic replay.

| Style | In | Out |
|-------|----|----|
| Flash | Bloom spike reveals scene | Flash bright, then black |
| Sweep | Energy beam sweeps, revealing | Energy beam sweeps, concealing |
| Glitch | Resolves from corruption | Corrupts to blackout |
| Collapse/Rebuild | Tiles build outward from center | Tiles collapse inward to center |

~0.3–0.5s each. Extensible via `TransitionStyle` enum.

### Collapse/Rebuild Detail

**Tile-based screen effect** — a post-processing shader, not per-entity animation. The screen is divided into a grid of tiles.

**Out (Collapse):** Tiles shrink + slide toward screen center in a radial wave (edges first, center last). Each tile rotates slightly as it moves. When all tiles converge: brief flash, then black.

**In (Rebuild):** Tiles expand outward from center to their grid positions. Center tiles first, edges last. Staggered wave gives a satisfying "explosion" reveal.

**Implementation:** `collapse_rebuild.wgsl` as a `FullscreenMaterial` on the camera. Uniforms: `progress` (0.0–1.0), `direction` (in/out), `tile_count` (grid dimensions), `tile_seed` (for slight per-tile timing variation from `GameRng`). The shader computes per-tile transform (position offset + scale + rotation) from progress and tile grid position. Tiles further from center (for Out) or closer to center (for In) animate first.

No game entities are touched — this is purely a screen-space visual effect that masks the state change beneath it.

## Implementation: screen/transition/

```
screen/
  transition/
    mod.rs              — TransitionPlugin, TransitionStyle enum, random selection
    flash.rs            — flash transition: drives TriggerScreenFlash via rantzsoft_vfx
    sweep.rs            — sweep transition: drives Beam + SparkBurst primitives
    glitch.rs           — glitch transition: drives ChromaticAberration + RadialDistortion + ScreenFlash
    collapse_rebuild.rs — collapse/rebuild: drives collapse_rebuild.wgsl FullscreenMaterial
```

Each transition style has a system that:
1. Reads the current transition state (time elapsed, direction In/Out)
2. Sends rantzsoft_vfx primitive messages (TriggerScreenFlash, TriggerChromaticAberration, etc.) to drive the visual
3. When the animation duration completes: sets the next PlayingState

**Random selection** uses `GameRng` (seeded from run seed) for deterministic replay.

**`TransitionComplete` message** sent by screen/ when the In animation finishes — consumed by any system that needs to know the transition is done.

## What Stays Where

| Concern | Owner |
|---------|-------|
| TransitionStyle enum, random selection | screen/ |
| Transition animation systems (flash.rs, sweep.rs, etc.) | screen/ |
| PlayingState substates | shared/ (GameState/PlayingState definitions) |
| TransitionComplete message | screen/ |
| VFX primitives used by transitions (ScreenFlash, ChromaticAberration, etc.) | rantzsoft_vfx |
| No transition code at all | rendering/ |

## No Separate Crate

Transitions stay in screen/. If a second game needs transition infrastructure, extract then. The design boundary: transition animation systems don't depend on game entity types (they drive camera-level VFX via the crate), so extraction would be clean.

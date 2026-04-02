# Research: Screen Wipe Transitions in Bevy 0.18

Bevy version: **0.18.1** (confirmed from workspace `Cargo.toml`).

---

## 1. Existing Crates for Screen Transitions

### bevy_screen_transitions

Does not exist on crates.io. No crate by this name (or `bevy-screen-transitions`) was found in any search. There is no dedicated community crate for Bevy screen transitions in the ecosystem.

### bevy_tweening 0.15.0

- **Bevy 0.18 compatible**: YES — version 0.15.0 explicitly states Bevy 0.18 support (released January 31, 2026).
- **Feature set for transitions**: Can tween `BackgroundColor` via `UiBackgroundColorLens { start: Color, end: Color }`, which interpolates all four channels including alpha. Also has `UiPositionLens` for animating `Node.left/top` for sweep-style effects.
- **What it adds over the current implementation**: Auto-completion callbacks (`TweenCompleted` event / `AnimCompletedEvent` via `EntityEvent` derive), easing curves (linear, quadratic, cubic, etc.), and sequence chaining. The current project implementation (`animate_transition`) uses a manual timer + lerp in `Update` — `bevy_tweening` replaces this with a component-driven approach.
- **What it does NOT help with**: State machine wiring. `bevy_tweening` animates values; it does not drive state transitions. The `TweenCompleted` event is used to trigger `next_state.set(...)` from a system — the same pattern already used in `animate_transition`.
- **Maintenance**: Active. 174+ contributors, updated to each Bevy release shortly after it ships.
- **Verdict**: Usable, but provides marginal benefit over the current manual implementation. The current code is simpler and has zero dependencies. Only adopt if easing curves or sequence chaining are needed.

### bevy_tween 0.12.0

- **Bevy 0.18 compatible**: YES — README version table explicitly maps bevy 0.18 → bevy_tween 0.12.
- **Feature set**: Functional/declarative animation framework, inspired by functional languages. More composable than `bevy_tweening` but with a steeper learning curve.
- **Screen transitions**: No examples provided. Focuses on individual entity property tweening.
- **Verdict**: No meaningful advantage over `bevy_tweening` for this use case. More complex API.

### Conclusion on crates

Neither external crate offers meaningful benefits over the project's existing `fx/transition/` implementation. The current system already handles Flash and Sweep styles, random selection via `GameRng`, RON-configurable durations, and correct state advancement. Adding a tweening crate would add a dependency for features not yet needed.

---

## 2. Common Patterns Without Crates

### Pattern A: Full-screen UI Node Overlay (what this project uses)

Spawn a `Node { width: 100%, height: 100%, position_type: Absolute }` with `BackgroundColor` during a dedicated transition state. Animate alpha (fade) or `node.left` (sweep) each `Update` tick. Despawn on `OnExit`.

**Key finding for this project**: The current implementation in `fx/transition/system.rs` already follows this pattern correctly. The `TransitionTimer` component drives animation, and `animate_transition` advances the state machine on completion.

**Z-order concern**: The existing code does NOT set `GlobalZIndex`. UI nodes without `GlobalZIndex` use local `ZIndex(0)`. If game HUD elements or other UI nodes share the same parent root, the overlay may render behind them.

**Fix**: Add `GlobalZIndex(i32::MAX - 1)` to the spawned overlay node. This is the same pattern used by Bevy's own FPS overlay (`FPS_OVERLAY_ZINDEX = i32::MAX - 32`). Verified: `GlobalZIndex(pub i32)` is a valid component in Bevy 0.18 (`bevy::ui::GlobalZIndex`), confirmed from `docs.rs/bevy/0.18.0`.

```rust
commands.spawn((
    TransitionOverlay,
    TransitionTimer { ... },
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    },
    BackgroundColor(color),
    GlobalZIndex(i32::MAX - 1),  // renders above all other UI nodes globally
));
```

### Pattern B: Sprite Overlay (not used here)

Use a sprite or mesh covering the screen. Provides Z-order control against world objects. However, does NOT overlay UI nodes (the UI camera renders on a separate pass at fixed depth). For a pure 2D brickbreaker where the transition should cover the entire screen including HUD, this pattern is worse than the UI node approach.

### Pattern C: Render Target / Camera (advanced, not needed)

Use a separate camera rendering to a texture, apply a shader effect. Complex, appropriate for shader-based wipes (e.g., iris-in, pixel dissolve). Overkill for fade/sweep. Not recommended unless the design specifically requires GPU-driven wipe effects.

### Pattern D: `FullscreenMaterial` Post-Process (shader-driven)

Use the project's already-verified `FullscreenMaterial` (see `agent-memory/researcher-bevy-api/rendering-fullscreen-material.md`) as a camera post-process effect. The fragment shader can implement any wipe pattern (radial, iris, pixel-shuffle, etc.). The material's uniform values (progress float) are driven by the `TransitionTimer` component on the camera entity or a separate resource.

**Advantage**: Pixel-perfect effects impossible with UI overlay (radial wipes, dissolve patterns, custom shader art). Full HDR support.

**Disadvantage**: Does NOT render on top of UI — post-process runs before UI compositing. For effects that must cover the HUD, the UI node approach (Pattern A) is required.

**Hybrid approach**: Use `FullscreenMaterial` for the visual effect (behind UI), plus a `BackgroundColor(Color::BLACK)` UI node with `GlobalZIndex(i32::MAX - 1)` to also cover HUD at the appropriate moment. This is complex and probably not needed for Flash/Sweep.

---

## 3. Transient State Pattern

The question asks whether a "transitioning" state can know its source and destination without requiring a `Transitioning` variant in every state enum.

### What Bevy 0.18 provides

`ComputedStates` can derive a new state from a tuple of source states. `StateTransitionEvent<S>` carries `exited: Option<S>` and `entered: Option<S>` fields. These are the two tools for a transient transition state.

### Option A: Explicit `TransitionOut` / `TransitionIn` variants (current approach)

The project already uses this: `GameState::TransitionOut` and `GameState::TransitionIn` are top-level `GameState` variants. The transition system spawns overlays `OnEnter`, animates in `Update`, and advances state on timer completion.

**Verdict**: Correct and simple. The state machine encodes transition intent directly. There is no need to "know where it came from" at the Bevy state level — the `TransitionDirection` field on `TransitionTimer` carries that information.

### Option B: `ComputedStates` for a generic `IsTransitioning` marker

```rust
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct IsTransitioning;

impl ComputedStates for IsTransitioning {
    type SourceStates = GameState;
    fn compute(source: GameState) -> Option<Self> {
        match source {
            GameState::TransitionOut | GameState::TransitionIn => Some(IsTransitioning),
            _ => None,
        }
    }
}
```

Systems that want to run during ANY transition regardless of direction can use `run_if(in_state(IsTransitioning))` instead of `.or(in_state(GameState::TransitionIn))`. This is a simplification convenience, not a new capability.

**Verdict**: Useful if many systems need to run during both transition phases. The project currently only has `animate_transition` spanning both phases (already using `.or()`), so the benefit is marginal right now.

### Option C: Storing source/destination in a Resource

For transitions that need to know "where they came from and where they're going" (e.g., to select an effect specific to the pair `Node → ChipSelect` vs `ChipSelect → Node`):

```rust
#[derive(Resource)]
struct PendingTransition {
    from: GameState,
    to: GameState,
    style: TransitionStyle,
}
```

Set this resource before calling `next_state.set(GameState::TransitionOut)`. The `spawn_transition_out` system reads it to pick a style or duration appropriate for the specific transition pair. Clear on `OnExit(GameState::TransitionIn)`.

**Verdict**: The cleanest pattern for "transitions that know their endpoints". No changes to the state enum. No `ComputedStates` needed. The project already uses a simpler version of this — `TransitionTimer` on the overlay entity carries the direction. Extend to `PendingTransition` resource if the transition needs to know both endpoints at spawn time.

### Option D: SubStates for "transitioning within RunState"

The question mentions wanting transitions at the `RunState` level (e.g., `NodeState → ChipSelect`). The project's current state hierarchy (`GameState::TransitionOut/In` at the top level) is correct for the inter-node flow. If transitions at the `RunState` level are desired instead:

```rust
#[derive(SubStates, Default, Clone, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Playing)]
enum RunState {
    // ...existing variants...
    TransitionOut,  // add here instead of at GameState level
    TransitionIn,
}
```

However, doing this would break the current pattern where `Playing` must be exited and re-entered to fire `OnEnter(Playing)` for node setup (see `docs/architecture/state.md`). The architecture doc explicitly explains: "The full inter-node flow is: `Playing → TransitionOut → ChipSelect → TransitionIn → Playing`. [...] Since node spawn/cleanup relies on `OnEnter(Playing)` / `OnExit(Playing)`, advancing between nodes requires leaving and re-entering `Playing`."

**Verdict**: Do not move transitions to a SubState of `Playing`. The current top-level `TransitionOut`/`TransitionIn` variants are architecturally correct.

---

## 4. Run Conditions During Transitions

### Question: Can systems from the "old" state keep rendering (frozen) while the transition plays?

In Bevy, when `NextState` is set, the state change applies at the END of the current frame (during the `StateTransition` schedule, which runs between `PreUpdate` and `Update`). So:

- Frame N: `next_state.set(TransitionOut)` is called
- Frame N end: State transitions to `TransitionOut`, `OnExit(Playing)` fires, `OnEnter(TransitionOut)` fires (spawns overlay)
- Frame N+1+: Only systems with `.run_if(in_state(GameState::TransitionOut))` run

This means the old state's visuals are despawned by the time the transition overlay first renders in earnest. There is no built-in "keep the old scene frozen behind the overlay" mechanism.

### Workarounds if frozen-behind-overlay is desired

**1. Cleanup-on-TransitionIn instead of cleanup-on-TransitionOut** (already the current approach): The playing field entities persist through `TransitionOut` and `ChipSelect`. Cleanup happens `OnEnter(TransitionIn)` or `OnEnter(Playing)`. The overlay obscures the old scene without despawning it. This is the right approach — verify whether cleanup timing is currently correct.

**2. Render to texture + crossfade**: Capture the last Playing frame to a texture before exiting, display as a static sprite behind the overlay during transition. Complex; not recommended for this project.

**3. Pause-based approach**: Instead of transitioning state, toggle a `GamePaused` resource and overlay. Old systems keep running but game logic is gated. The overlay fades in/out. Used for pause menus; unsuitable for inter-node transitions because the old node must be cleaned up.

### What the current project does

The `OnExit(GameState::TransitionOut)` and `OnExit(GameState::TransitionIn)` call `cleanup_transition` (despawns the overlay entity). The playing field cleanup must happen either in `OnExit(Playing)` or `OnEnter(TransitionIn)` (as described in the architecture doc). The overlay covers whatever is on screen during `TransitionOut` — if the playing field is still present, it is obscured. This is the correct pattern and requires no changes.

---

## 5. Bevy 0.18 State API — Confirmed Details

### `StateTransitionEvent<S>` (Bevy 0.18)

```rust
pub struct StateTransitionEvent<S: States> {
    pub exited: Option<S>,
    pub entered: Option<S>,
    pub allow_same_state_transitions: bool,
}
```

Implements `Message`. Fires AFTER the transition completes (after `OnEnter`/`OnExit` run). Useful for detecting completed transitions from systems that need to react.

**Bevy 0.18 breaking change**: `next_state.set(S)` now ALWAYS fires `OnEnter`/`OnExit`, even when setting the same state value. Use `next_state.set_if_neq(S)` for the old behavior.

### `OnTransition<S>` schedule

Runs for any transition between two states. Takes `from` and `to` variants:

```rust
// Runs when transitioning from TransitionOut to ChipSelect:
app.add_systems(OnTransition { from: GameState::TransitionOut, to: GameState::ChipSelect }, my_system);
```

Less commonly used than `OnEnter`/`OnExit`. Useful for one-time setup that needs both the old and new state context.

---

## 6. Summary and Recommendations for RunState Transitions

The question asks about transitions between `RunState` changes (e.g., `Node → ChipSelect → Node`). Based on the architecture doc and the current implementation:

### Current state

The project already has a working, tested transition system:
- `GameState::TransitionOut` → spawns overlay `OnEnter`, animates in `Update`, transitions to `GameState::ChipSelect` on completion
- `GameState::ChipSelect` → chip selection UI
- `GameState::TransitionIn` → spawns overlay `OnEnter`, animates in `Update`, transitions to `GameState::Playing` on completion
- Two `TransitionStyle` variants: `Flash` (alpha fade) and `Sweep` (horizontal rect sweep)
- Random style selection via `GameRng`
- RON-configurable durations and colors (`TransitionConfig`)

### Gaps to address

**Gap 1 — `GlobalZIndex` missing on overlay**: The spawned overlay node in `spawn_transition_out` and `spawn_transition_in` does not include `GlobalZIndex`. Without it, the overlay may render behind HUD elements. Add `GlobalZIndex(i32::MAX - 1)` to both spawn calls.

**Gap 2 — No `ZIndex` / `GlobalZIndex` for SWEEP style**: The sweep effect moves `node.left`, which works for the overlay background but does not clip child elements if any exist. Not a current issue (the overlay is a simple colored node), but worth noting.

**Gap 3 — Transition style selection is binary (50/50)**: Currently `pick_style` uses `rng.0.random_range(0..2)`. To add more styles, extend `TransitionStyle` and update `pick_style` to weight from a configurable pool. The `TransitionConfig` RON can carry a `style_weights` array.

### Adding more transition styles

Each new style requires:
1. A new `TransitionStyle` variant
2. A match arm in `animate_transition` for the animation logic
3. A match arm in `overlay_color` for initial alpha
4. Updated `pick_style` or a weighted selection system

The current architecture supports this without structural changes. New styles that require shader effects (iris wipe, pixel dissolve) would use the `FullscreenMaterial` post-process path instead of the UI overlay path — but these cannot cover UI. For a brickbreaker where the primary content is the play field (not behind the overlay's scope), this may be acceptable.

### No external crate needed

The existing implementation covers the stated requirements. The only actionable fix is adding `GlobalZIndex` to ensure the overlay covers all UI layers.

---

## Quick Reference

| Component | Module path | Use |
|-----------|-------------|-----|
| `GlobalZIndex(i32)` | `bevy::ui::GlobalZIndex` (re-exported in `bevy::prelude`) | Render UI node above all other UI nodes |
| `BackgroundColor(Color)` | `bevy::prelude::BackgroundColor` | Background color of a UI node, animatable |
| `StateTransitionEvent<S>` | `bevy::state::state::StateTransitionEvent` | Message fired after any state transition |
| `UiBackgroundColorLens` | `bevy_tweening::lens::UiBackgroundColorLens` | (if adopting bevy_tweening) Lerp BackgroundColor |

| Crate | Bevy 0.18 support | Verdict |
|-------|-------------------|---------|
| `bevy_tweening` 0.15.0 | YES | Usable but not needed; adds easing curves, completion events |
| `bevy_tween` 0.12.0 | YES | More complex API, no screen transition examples |
| `bevy_screen_transitions` | Does not exist | N/A |

---

## Sources

Verified against:
- `docs.rs/bevy/0.18.0` — `ComputedStates`, `SubStates`, `GlobalZIndex`, `StateTransitionEvent`, `AppExtStates`
- `github.com/bevyengine/bevy/blob/v0.18.0/examples/state/computed_states.rs`
- `github.com/bevyengine/bevy/blob/v0.18.0/examples/state/sub_states.rs`
- `docs.rs/bevy_tweening/0.15.0` — lens types, Bevy 0.18 compatibility
- `docs.rs/bevy_dev_tools/0.18.1/src/bevy_dev_tools/fps_overlay.rs` — GlobalZIndex usage pattern
- `bevy.org/learn/migration-guides/0-17-to-0-18/` — breaking changes relevant to transitions
- Project source: `breaker-game/src/fx/transition/`, `breaker-game/src/shared/game_state.rs`, `docs/architecture/state.md`

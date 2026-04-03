# Writing a Transition

How to implement a new transition effect for the `rantzsoft_lifecycle` crate. Covers both built-in effects and custom effects registered by the game.

## What a Transition Is

A transition is three Bevy systems (start, run, end) that animate a visual effect while the crate orchestrates state changes. The crate drives the lifecycle via marker resources — your systems just react to those markers.

## Required Pieces

### 1. Marker Struct

A struct that holds the effect's parameters. Implements one or more marker traits.

```rust
pub struct MyWipeOut {
    pub color: Color,
    pub config: TransitionConfig,
}

impl Transition for MyWipeOut {}
impl OutTransition for MyWipeOut {}
```

Marker traits to implement:
- `InTransition` — can reveal a screen (transparent overlay → removed)
- `OutTransition` — can cover a screen (nothing → opaque overlay)
- `OneShotTransition` — both screens coexist during effect (no overlay)
- Implement multiple if the effect works in both directions (rare — usually you have separate In/Out structs)

### 2. Three Systems

```rust
fn my_wipe_out_start(
    mut commands: Commands,
    time: Res<Time<Real>>,
    transition: Res<StartingTransition<MyWipeOut>>,
    // ... spawn overlay, record start time
) {
    // Spawn overlay entity:
    // - GlobalZIndex(i32::MAX - 1) so it renders above everything
    // - Full viewport size (query the camera for dimensions)
    // - Initial state (fully transparent for Out, fully opaque for In)

    // Send TransitionReady when setup is complete
}

fn my_wipe_out_run(
    time: Res<Time<Real>>,
    transition: Res<RunningTransition<MyWipeOut>>,
    // ... query overlay entity, animate
) {
    // Each frame:
    // 1. Compute progress = elapsed / duration (using Time<Real>)
    // 2. Sample easing curve at progress → get 0.0–1.0 value
    // 3. Apply value to overlay (alpha, position, uniform, etc.)
    // 4. When progress >= 1.0 → send TransitionRunComplete
}

fn my_wipe_out_end(
    mut commands: Commands,
    transition: Res<EndingTransition<MyWipeOut>>,
    // ... query overlay entity
) {
    // Despawn overlay entity (for Out: leave nothing, screen is covered by state change)
    // For In: despawn overlay, screen is now visible
    // Send TransitionOver
}
```

### 3. Registration

Built-in transitions are registered inside the `LifecyclePlugin`. Custom transitions use the plugin builder:

```rust
app.add_plugin(LifecyclePlugin::new()
    .register_custom_transition::<MyWipeOut>(
        my_wipe_out_start,
        my_wipe_out_run,
        my_wipe_out_end,
    )
);
```

## Rules

### Time<Real>, NEVER Time<Virtual>

Transitions run while virtual time is paused. All elapsed time calculations MUST use `Time<Real>`. Using `Time<Virtual>` (or the default `Time` in `Update`) will freeze the animation.

### GlobalZIndex(i32::MAX - 1)

All overlay-based transitions (everything except camera-based effects like Slide) MUST spawn their overlay entity at `GlobalZIndex(i32::MAX - 1)`. This ensures the overlay renders above all game content but below debug UI.

### Viewport Resize

The overlay must stay full-viewport if the window resizes mid-transition. Re-derive overlay size from the camera projection each frame in the `run` system, not just in `start`.

### OutIn Duration Split

`TransitionConfig.duration` is the TOTAL duration for OutIn variants. Each phase (Out and In) gets half. The crate handles this split — individual In/Out systems always receive the per-phase duration.

### Easing Curve

The easing curve maps elapsed time to a 0.0–1.0 progress value. Your `run` system samples it and applies the result. The curve is in `TransitionConfig` — don't hardcode easing behavior.

### No Game Knowledge

Transition systems must not reference any game types, components, or resources. They operate on their own overlay entities and the crate's marker resources only. This is a `rantzsoft_*` crate — zero game vocabulary.

### Cleanup

For standalone Out, standalone In, and OneShot: `end` systems MUST despawn all entities they spawned.

For OutIn: **the crate handles overlay handoff** — effect `end` systems do NOT despawn the overlay. See "OutIn Overlay Handoff" below.

## OutIn Overlay Handoff

Simple two-step handoff — each effect phase is fully self-contained:

1. **Out `end` skips despawn** — overlay stays, screen remains covered
2. **Crate despawns the Out overlay** — screen is still covered by the state change
3. **In `start` spawns its own overlay fresh** — at full opacity, its own color/material
4. **In `end` despawns normally**

No entity passing between phases. No material swapping. Each effect owns its own overlay lifecycle — the only difference in OutIn mode is that Out skips its despawn.

```rust
pub struct EndingTransition<T: Transition> {
    pub effect: T,
    /// If true, crate will despawn your overlay after your `end` runs.
    /// Your `end` system should NOT despawn it.
    /// If false, you're standalone — despawn your entities in `end`.
    pub crate_owns_overlay: bool,
}
```

**Mixed-type OutIn color mismatch:**

If the game routes `FadeOut(Color::BLACK)` → `WipeIn(Color::PINK)`, there's a single frame between Out overlay despawn and In overlay spawn where the new state is briefly visible. At normal transition speeds this is imperceptible — both overlays are at full opacity. But mismatched colors produce a hard cut from solid black to solid pink. This is the game's problem — use matching colors for seamless OutIn transitions.

## Example: Full FadeOut Implementation

```rust
// ── Marker struct ────────────────────────────────────
pub struct FadeOut {
    pub color: Color,
    pub config: TransitionConfig,
}
impl Transition for FadeOut {}
impl OutTransition for FadeOut {}
impl Default for FadeOut {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            config: TransitionConfig::default(), // 0.3s, EaseOutCubic
        }
    }
}

// ── Overlay marker component ─────────────────────────
#[derive(Component)]
struct FadeOverlay;

// ── Start ────────────────────────────────────────────
fn fade_out_start(
    mut commands: Commands,
    transition: Res<StartingTransition<FadeOut>>,
    camera_q: Query<&OrthographicProjection, With<Camera2d>>,
) {
    let projection = camera_q.single();
    let size = projection.area.size();

    // Always spawn — each phase owns its own overlay
    commands.spawn((
        Sprite {
            color: transition.effect.color.with_alpha(0.0),
            custom_size: Some(size),
            ..default()
        },
        GlobalZIndex(i32::MAX - 1),
        FadeOverlay,
    ));

    // Signal: setup complete, start animating
    // (sends TransitionReady)
}

// ── Run ──────────────────────────────────────────────
fn fade_out_run(
    time: Res<Time<Real>>,
    transition: Res<RunningTransition<FadeOut>>,
    camera_q: Query<&OrthographicProjection, With<Camera2d>>,
    mut overlay_q: Query<(&mut Sprite,), With<FadeOverlay>>,
) {
    let progress = (time.elapsed_secs_f64() - transition.start_time)
        / transition.effect.config.duration.as_secs_f64();
    let progress = progress.clamp(0.0, 1.0);

    // Sample easing curve
    let eased = transition.effect.config.easing.sample(progress as f32);

    // Update overlay alpha
    let (mut sprite,) = overlay_q.single_mut();
    sprite.color = transition.effect.color.with_alpha(eased);

    // Re-derive size from camera each frame (handles viewport resize)
    let projection = camera_q.single();
    sprite.custom_size = Some(projection.area.size());

    if progress >= 1.0 {
        // Signal: animation complete
        // (sends TransitionRunComplete)
    }
}

// ── End ──────────────────────────────────────────────
fn fade_out_end(
    mut commands: Commands,
    transition: Res<EndingTransition<FadeOut>>,
    overlay_q: Query<Entity, With<FadeOverlay>>,
) {
    if !transition.crate_owns_overlay {
        // Standalone mode — we own the overlay, despawn it
        for entity in &overlay_q {
            commands.entity(entity).despawn();
        }
    }
    // OutIn mode — leave overlay for the crate to hand off to In phase

    // Signal: cleanup complete
    // (sends TransitionOver)
}
```

## Shader-Based Transitions

Dissolve, Iris, and Pixelate use custom shader materials instead of plain sprites. The pattern is the same — spawn an overlay entity with a custom `Material2d`, update uniforms in the `run` system, despawn in `end`. The shader does the visual work; the system just feeds it a progress value.

Shader files live in the crate's `assets/shaders/` directory (e.g., `dissolve.wgsl`, `iris.wgsl`, `pixelate.wgsl`).

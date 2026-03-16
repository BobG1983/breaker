# Bevy 0.18.1 — Dangerous Gotchas (Quick Reference)

This file is a safety net for patterns where training data confidently produces wrong code.
It is NOT a full API guide. Verify edge cases at docs.rs/bevy/0.18.1.

---

## 1. Breaking Renames

| Old (training data says) | Correct in 0.18.1 |
|--------------------------|-------------------|
| `Parent` component | `ChildOf` — `ChildOf(pub Entity)`, method `.parent()` |
| `bevy_hierarchy` crate | Merged into `bevy_ecs` — import from `bevy::prelude` |
| `InputSystem` system set | `InputSystems` (plural) |
| `bevy::core_pipeline::bloom::Bloom` | `bevy::post_process::bloom::Bloom` |

## 2. Removed / Replaced APIs

- **NO `*Bundle` types** — `SpriteBundle`, `NodeBundle`, `CameraBundle`, etc. do not exist.
  Spawn with component tuples: `commands.spawn((Camera2d, Transform::default()))`.
- **NO `EventReader<KeyboardInput>`** — `KeyboardInput` is a `Message`, not an `Event`.
  Use `MessageReader<KeyboardInput>`. Same for all input messages.
- **NO `app.world_mut().send_event()`** — for Messages use `world.write_message(msg)`.
- **NO `EventWriter` for game communication** — use `MessageWriter<T>` + `app.add_message::<T>()`.
- **NO `WindowMode::Maximized`** — use `window.set_maximized(true)` method instead.
- **`#[derive(Event)]` is for observer/triggered events ONLY** — game messages use `#[derive(Message)]`.

## 3. Silent Behavior Changes (no compile error, wrong runtime behavior)

- **`advance_by` does NOT trigger `FixedUpdate`** — it moves the clock but skips the overstep
  accumulator. `run_fixed_main_schedule` reads the accumulator. Use `accumulate_overstep(timestep)`
  in tests to trigger FixedUpdate ticks.
- **`commands.entity(id).despawn()` is recursive** — despawns the entity AND all descendants.
  There is no `despawn_recursive` in 0.18.1; plain `despawn()` already does this.
- **z-scale of 2D sprites/meshes MUST be 1.0** — non-unit z-scale breaks render ordering silently.
- **`ChildOf` component lives on the CHILD** — `Children` lives on the parent. Never manually
  mutate `Children`; hierarchy hooks maintain it automatically.
- **Input clearing schedule** — clear input state in `FixedPostUpdate`, NOT `PreUpdate`. Clearing in
  `PreUpdate` loses inputs on frames where `FixedUpdate` is skipped.

## 4. Test-Critical Patterns

```rust
// Trigger FixedUpdate in a test (CORRECT):
let timestep = app.world().resource::<Time<Fixed>>().timestep();
app.world_mut().resource_mut::<Time<Fixed>>().accumulate_overstep(timestep);
app.update();

// Send a keyboard message in a test (CORRECT):
let window = app.world_mut().spawn_empty().id();
app.world_mut().write_message(KeyboardInput {
    key_code: KeyCode::ArrowLeft,
    logical_key: Key::ArrowLeft,
    state: ButtonState::Pressed,
    text: None,
    repeat: false,
    window,
});
app.update();
```

## 5. Common Import Gotchas

- `KeyboardInput`, `Key`, `ButtonState` are NOT in `bevy::prelude` — import from
  `bevy::input::keyboard::{Key, KeyboardInput}` and `bevy::input::ButtonState`.
- `KeyCode` and `ButtonInput` ARE in `bevy::prelude`.
- `Bloom` is at `bevy::post_process::bloom::Bloom` (also re-exported from prelude).
- `Tonemapping::TonyMcMapface` requires the `tonemapping_luts` Cargo feature — NOT included in
  `features = ["2d"]`. Safe alternatives: `Reinhard`, `ReinhardLuminance`, `AcesFitted`.
- `OrthographicProjection::default_2d()` — sets `near` to a negative value. Do NOT use
  `OrthographicProjection::default()` for 2D cameras; it uses wrong near plane.
- System output must be `()` — use `.map(drop)` to discard non-unit return values (e.g., `Progress`).
  There is no `.ignore()` helper.

## 6. Spawn Patterns (correct for 0.18.1)

```rust
// Camera:
commands.spawn(Camera2d);  // Required components auto-inserted

// Solid-color rectangle (no asset needed):
commands.spawn((
    Sprite::from_color(Color::WHITE, Vec2::ONE),
    Transform { scale: Vec3::new(w, h, 1.0), translation: pos.extend(z), ..default() },
));

// Hierarchy:
commands.entity(parent).with_child(bundle);  // auto-inserts ChildOf
// OR: commands.spawn((MyComp, ChildOf(parent_entity)));
```

---
name: Confirmed Patterns
description: Bevy 0.18.1 API patterns confirmed correct for this codebase
type: reference
---

## Bevy Version
Bevy 0.18.1, `features = ["2d"]`, `default-features = false`

## Hierarchy
- `ChildOf` (not `Parent`) is the parent component
- `child_of.parent()` method is correct
- `commands.entity(e).with_child(bundle)` auto-inserts `ChildOf`

## Despawn
- `commands.entity(e).despawn()` is RECURSIVE in 0.18 (`despawn_recursive` removed)

## Query API
- `query.single()` returns `Result<D, QuerySingleError>` — fallible
- `let Ok(...) = query.single() else { return; }` pattern is correct
- `query.single_mut()` also returns a Result

## Messages
- `#[derive(Message)]` + `MessageWriter<T>` + `MessageReader<T>` + `app.add_message::<M>()` — correct
- `AppExit` implements `Message` — `MessageWriter<AppExit>` valid
- `KeyboardInput` is a `Message` — `MessageReader<KeyboardInput>` correct
- `world.write_message(msg)` for tests
- `writer.write(msg)` — correct method name (NOT `send`)
- `reader.read()` returns `MessageIterator` yielding `&'a M`

## Spawn Patterns
- `Mesh2d(meshes.add(...))` + `MeshMaterial2d(materials.add(...))` — no bundles
- `Camera2d` zero-size marker with required components
- `Sprite::from_color()` / `ColorMaterial::from_color()`
- `Circle::new(r)` + `Rectangle::new(w, h)` in prelude

## UI (no bundles)
- `Node { ... }` directly (not `NodeBundle`)
- `Text::new("...")` for UI text, `Text2d::new("...")` for world text
- `TextFont { font_size, ..default() }` / `TextFont::from_font_size(f32)`
- `TextColor(Color::...)`, `BackgroundColor(Color::...)`, `BorderColor::all(...)`
- `Button` as marker component

## Gizmos API
- `gizmos.circle_2d(impl Into<Isometry2d>, radius, color)` — Vec2 works
- `gizmos.rect_2d(impl Into<Isometry2d>, Vec2, color)`
- `gizmos.arrow_2d(Vec2, Vec2, color)` — takes Vec2 directly

## State API
- `#[derive(States)]`, `app.init_state::<S>()`, `in_state(S::Variant)`
- `#[derive(SubStates)]` with `#[source(ParentState = ParentState::Variant)]`
- `OnEnter(S::V)`, `OnExit(S::V)` schedule labels

## EguiPlugin
- `EguiPlugin::default()` — bevy_egui 0.39
- Debug UI in `EguiPrimaryContextPass` schedule

## Fixed Update Testing
- `accumulate_overstep(timestep)` triggers FixedUpdate (NOT advance_by)

## Easing
- `EaseFunction::QuadraticIn` etc. in `bevy::math::curve::easing`
- `.sample_clamped(t)` on `EaseFunction` (implements `Curve<f32>`)

## Input
- `Res<ButtonInput<KeyCode>>` with `.pressed()`, `.just_pressed()`
- `InputSystems` (plural) system set

## Camera
- `Projection::from(OrthographicProjection { ... })`, `OrthographicProjection::default_2d()`
- `Tonemapping::AcesFitted` — safe (no LUT)

## Window
- `window.set_maximized(true)` — no `WindowMode::Maximized`

## bevy_common_assets 0.15
- `RonAssetPlugin::<T>::new(&[...])` accepts multiple extensions

## bevy_asset_loader 0.25
- `#[asset(path = "folder", collection(typed))]` on `Vec<Handle<T>>`

## Schedule Ordering
- `.after(fn_from_another_plugin)` for cross-plugin ordering — correct, fn pointers implement IntoSystemSet

## ApplyDeferred
- `(system_a, ApplyDeferred, system_b).chain()` — correct; flushes commands between systems
- The "does nothing" warning only applies to `.pipe()`, NOT `.chain()`

## Node Fields
- `Node::row_gap: Val`, `Node::column_gap: Val` — confirmed

## EntityCommands
- `commands.entity(e).insert_if_new(bundle)` — confirmed; leave-old semantics
- Tuple bundles work with `insert_if_new`

## f32::midpoint
- Stable since Rust 1.85.0; project uses edition 2024 (requires 1.85.0+)

## ResMut<GameRng>
- Valid system parameter; `init_resource::<GameRng>()` in tests

## Query tuple limits
- Up to 15 elements (QueryData derive on tuples via macro)

## World::get_entity
- `world.get_entity(e)` returns `Result<EntityRef, Entity>` in 0.18
- `.is_ok()` / `.is_err()` are correct ways to test existence in tests

## Has<T> in queries
- `Has<T>` as a query element returns `bool` — correct QueryData for 0.18
- Used in type alias query tuples: `(Entity, &Component, Has<Marker>)` — confirmed correct
- Lives in `bevy::ecs::query::Has`

## cfg_attr(test, allow(...)) with reason
- `#[cfg_attr(test, allow(clippy::unwrap_used, ..., reason = "..."))]` in lib.rs — correct
- This is a conditional allow (only in test builds) with a reason; satisfies `allow_attributes_without_reason`
- NOT a bare `#[allow]` — this pattern is project-approved for test assertions in lib.rs

## Option<ResMut<T>> system parameter
- `Option<ResMut<T>>` is a valid system parameter — `None` when resource not yet inserted
- Used safely in scenario runner for optional resource presence

## World resource access (post-run, outside systems)
- `app.world().get_resource::<T>()` → `Option<&T>` — correct for Bevy 0.18
- `app.world_mut().get_resource_mut::<T>()` → `Option<Mut<T>>` — correct for Bevy 0.18
- Both patterns are used in scenario runner's `collect_and_evaluate` and `drain_remaining_logs`
- `app.world().resource::<T>()` → `&T` (panics if missing) — also correct, used in tests

## add_message idempotency
- `app.add_message::<T>()` is idempotent — `SubApp::add_message` guards with `contains_resource::<Messages<T>>()`
- Calling it for the same type in both `Game` plugin and `ScenarioLifecycle` is safe — second call is a no-op
- Source: `bevy_app-0.18.1/src/sub_app.rs` lines 353-363

## Messages<T> direct world access
- `world.resource_mut::<Messages<T>>().write(msg)` — valid for test message injection
- `world.resource::<Messages<T>>().iter_current_update_messages()` — valid for test message assertion
- Both confirmed on docs.rs/bevy/0.18.1 `Messages` struct page

## Tuple SystemParam with multiple ResMut
- `(ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>)` — valid system parameter in Bevy 0.18.1
- Tuples up to 17 items implement SystemParam; two different ResMut types have no conflict
- Using `.0` and `.1` to access each from inside the system — correct

## init_resource with manual Default
- `app.init_resource::<T>()` requires `T: Resource + Default`
- Manual `impl Default` satisfies this — does not require `#[derive(Default)]`
- Confirmed: `ScenarioVerdict` uses manual `Default` impl, `init_resource` is valid

## Time<Real>
- `Res<Time<Real>>` is valid; `.elapsed_secs_f64()` method confirmed correct

## TimeUpdateStrategy
- `app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(16)))` — confirmed correct for test time control
- `bevy::time::TimeUpdateStrategy` import path correct

## StatesPlugin
- `bevy::state::app::StatesPlugin` — correct import for adding state machine support in minimal test apps

## FixedPreUpdate Schedule
- `FixedPreUpdate` is a valid schedule label in Bevy 0.18.1 (`pub struct FixedPreUpdate`)
- Runs before `FixedUpdate` within the `FixedMain` group
- `.add_systems(FixedPreUpdate, my_system)` — correct
- Appropriate for input injection that must arrive before FixedUpdate game systems read it

## NextState API
- `NextState<S>` is an **enum** (not a struct) in Bevy 0.18.1
- Variants: `Unchanged`, `Pending(S)`, `PendingIfNeq(S)`
- `next_state.set(S::Variant)` — correct; triggers `OnEnter`/`OnExit` schedules
- `next_state.set_if_neq(S::Variant)` — skips transition schedules if same state
- Used as `ResMut<NextState<S>>` system parameter — correct

## in_state run condition
- `in_state(S::Variant)` — valid run condition for any schedule including `Update`
- `.add_systems(Update, my_system.run_if(in_state(GameState::RunEnd)))` — correct

## Headless MinimalPlugins + Manual Plugin Stack (scenario runner)

The following plugin combination is confirmed correct for Bevy 0.18.1 headless mode (`default-features = false, features = ["2d"]`):

```rust
app.add_plugins((
    MinimalPlugins,                          // TaskPoolPlugin, TimePlugin, ScheduleRunnerPlugin
    bevy::state::app::StatesPlugin,          // NOT included in MinimalPlugins — must be explicit
    bevy::asset::AssetPlugin { file_path, ..default() },
    bevy::input::InputPlugin,
    bevy::mesh::MeshPlugin,
));
app.init_asset::<ColorMaterial>();           // partial registration — gives Assets<ColorMaterial> without GPU pipeline
app.add_plugins(bevy::text::TextPlugin);    // zero RenderApp dependency, safe headless
```

- `bevy::state::app::StatesPlugin` — NOT in MinimalPlugins; this explicit add is required
- `bevy::input::InputPlugin` — correct re-export path (`bevy_input` → `bevy::input`)
- `bevy::mesh::MeshPlugin` — correct re-export path (`bevy_mesh` → `bevy::mesh`), available under `"2d"` feature
- `bevy::text::TextPlugin` — correct re-export; no RenderApp access, verified pure CPU
- `init_asset::<ColorMaterial>()` — valid; `AssetPlugin` is added first in tuple so AssetServer is live; partial registration intentional (no GPU extraction needed)
- `MinimalPlugins` includes `TaskPoolPlugin` + `TimePlugin` + `ScheduleRunnerPlugin` — confirmed

## Local<T> System Parameter
- `Local<T>` requires `T: FromWorld + Send + 'static`
- Blanket impl: `impl<T: Default> FromWorld for T` — so `Local<Vec<U>>` is valid and auto-initializes to `Vec::new()`
- Each system gets its own isolated instance; value persists across frames
- Fully composable with any other SystemParam: Commands, Query, Res, ResMut, MessageReader, MessageWriter, other Locals
- No ordering constraints relative to other params
- Pattern confirmed correct: `mut local: Local<Vec<(Entity, f32, f32, f32, bool)>>` with `.clear()` + `.extend()` + `.iter()` — idiomatic scratch-buffer pattern; reuses heap allocation after warmup

## Patterns That Look Wrong But Are Correct
- `commands.entity(e).despawn()` on UI roots with children — recursive in 0.18+
- `gizmos.circle_2d(vec2, ...)` — Vec2 implements Into<Isometry2d>
- `MessageWriter<AppExit>` — AppExit implements Message
- `(spawn_side_panels, ApplyDeferred, spawn_timer_hud).chain()` — ApplyDeferred works in .chain()
- `commands.entity(panel).with_children(...)` on existing entity — correct
- Cross-plugin ordering with `.after(fn_name)` — correct
- `Has<RequiredToClear>` in query tuple — correct, yields bool
- `world.get_entity(e).is_err()` after `commands.entity(e).despawn()` + tick — valid existence test in 0.18

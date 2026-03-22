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

## EguiContexts::ctx_mut() in bevy_egui 0.39
- Returns `Result<&mut Context, QuerySingleError>`
- `let Ok(ctx) = contexts.ctx_mut() else { return; }` — correct pattern
- Systems using this run in `bevy_egui::EguiPrimaryContextPass` schedule — correct

## bevy::platform::collections::HashMap
- Correct import path for Bevy 0.18.1's platform-aware HashMap
- Used in `Local<HashMap<Entity, BreakerState>>` in invariant checkers — confirmed

## ChildSpawnerCommands
- `bevy::ecs::hierarchy::ChildSpawnerCommands<'_>` — correct type in spawn_chip_select helper functions
- Import: `use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};`

## BackgroundColor(Color::NONE)
- `Color::NONE` is transparent; `BackgroundColor(Color::NONE)` — valid in 0.18

## BorderColor::all(color)
- `BorderColor::all(border_color)` — confirmed constructor in 0.18 UI API

## MessageWriter<T> inside Observers (confirmed 2026-03-19)
- `fn handle(trigger: On<E>, mut writer: MessageWriter<M>)` — valid; MessageWriter<T> is a SystemParam and composable in observer fns
- `fn handle(trigger: On<E>, mut query: Query<...>, mut writer: MessageWriter<M>)` — valid; all SystemParams compose with On<E>
- Pattern used in `behaviors/effects/life_lost.rs`, `spawn_bolt.rs`, `time_penalty.rs` (directory renamed consequences/→effects/ in refactor/unify-behaviors 2026-03-21)

## any_with_component run condition (confirmed 2026-03-19)
- `any_with_component::<T>` is in the Bevy prelude for 0.18.1
- Signature: `fn any_with_component<T>(query: Query<(), With<T>>) -> bool where T: Component`
- `.run_if(any_with_component::<LivesDisplay>)` — correct

## Observer Pattern (confirmed for this codebase, 2026-03-19)
- `fn handler(trigger: On<MyEvent>, mut query: Query<...>, mut commands: Commands)` — correct observer signature; On<E> plus arbitrary SystemParams is valid
- `app.add_observer(my_handler)` — correct app-level global observer registration in Plugin::build
- `commands.trigger(MyEvent { ... })` — correct deferred global trigger; observers run at command flush
- `world.commands().trigger(...)` — correct in tests (deferred; must call `world_mut().flush()` after)
- `#[derive(Event)]` on the trigger struct — correct; GlobalTrigger (not EntityEvent)
- Multiple distinct observer fns for the same `On<MyEvent>` type — all run, in registration order
- Observer fn with `Query<(Entity, Option<&mut C>), With<Marker>>` + `Commands` — confirmed valid combination

## #[derive(SystemParam)] with ResMut fields (confirmed 2026-03-19)
- `#[derive(SystemParam)] struct Foo<'w> { field: ResMut<'w, T> }` — correct; lifetime 'w required
- Multiple `ResMut` fields for DIFFERENT resource types in same SystemParam struct — valid, no conflict
- SystemParam struct used as system function parameter — correct; all fields extracted at scheduling time
- `confirm.field.set(...)` — correct deref through ResMut; NestedMut access works

## AssetEvent<A> as Message
- `AssetEvent<A>` derives `Message` in Bevy 0.18.1 — use `MessageReader<AssetEvent<A>>`
- NEVER use `EventReader<AssetEvent<A>>` — AssetEvent is not an Event in this version
- Confirmed in researcher memory (core-api.md line 62) and used throughout hot_reload systems

## SystemParam with Query + Commands (two lifetimes)
- `#[derive(SystemParam)] struct Foo<'w, 's>` — requires BOTH lifetimes when struct contains Query or Commands
- `Query<'w, 's, D, F>` and `Commands<'w, 's>` fields require the `'s` state lifetime
- `LayoutChangeContext<'w, 's>` and `ArchetypeChangeContext<'w, 's>` — both correct patterns

## Additional Confirmed Schedules
- `FixedFirst` — valid Bevy 0.18.1 schedule, runs before FixedUpdate in FixedMain group
- `FixedPostUpdate` — valid, runs after FixedUpdate in FixedMain group
- `Last` — valid for end-of-frame tasks (e.g., write_recording_on_exit)
- `EguiPrimaryContextPass` — correct schedule for bevy_egui 0.39 UI rendering

## Additional Confirmed Run Conditions
- `resource_exists::<T>` — valid run condition, in prelude for 0.18
- `resource_changed::<T>` — valid run condition, in prelude for 0.18
- `res.is_changed() && !res.is_added()` — correct change-detection-only-after-add pattern

## Additional Confirmed UI Patterns
- `BorderRadius::all(Val::Px(n))` — correct 0.18 UI API
- `UiRect::axes(horizontal, vertical)` — confirmed
- `UiRect::right(val)` / `UiRect::left(val)` — confirmed
- `.with_child(bundle)` — single-child shorthand, correct
- `Text::new("...")` then `.0` for field access — Text is a newtype with `.0: String`

## Additional Confirmed Asset Patterns
- `AssetServer::load::<Font>(&string_path)` — returns `Handle<Font>`, correct
- `init_asset::<T>()` + `init_asset::<ColorMaterial>()` in tests — correct partial registration

## &Entities System Parameter
- `&Entities` implements `SystemParam` and `ReadOnlySystemParam` in Bevy 0.18.1
- Used as `all_entities: &Entities` in systems for presence checks
- `all_entities.contains(entity)` — correct method to test entity existence

## App::should_exit
- `app.should_exit()` — returns `Option<AppExit>` in Bevy 0.18.1
- `.is_some()` — correct pattern to check if app has signaled exit

## App::finish and App::cleanup
- `app.finish()` then `app.cleanup()` before manual `.update()` loop — correct headless init sequence
- Required to initialize plugins before the manual update loop runs

## system.map(drop) for systems returning Progress
- Systems returning `iyes_progress::Progress` must use `.map(drop)` when added via `add_systems`
- `.add_systems(Update, seed_foo.map(drop))` — correct; discards the `Progress` return value

## Time<Real> elapsed_secs_f64
- `time.elapsed_secs_f64()` — confirmed correct f64 elapsed on `Res<Time<Real>>`
- Used in double-tap detection in `read_input.rs`

## world.write_message
- `app.world_mut().write_message(msg)` — correct test helper for injecting messages directly into world
- Used for `KeyboardInput` injection in tests

## Bloom + Tonemapping in game
- `post_process::bloom::Bloom` — correct import path for `"2d"` feature
- `Tonemapping::AcesFitted` — safe (no LUT), confirmed in use
- `Bloom::default()` — valid preset in 0.18.1

## SubStates
- `#[derive(SubStates)]` with `#[source(GameState = GameState::Playing)]` — correct
- `app.add_sub_state::<PlayingState>()` — correct registration (idempotent; multiple plugins call it)

## Interpolation schedule usage
- `FixedFirst` for `restore_authoritative` — correct (before FixedUpdate)
- `FixedPostUpdate` for `store_authoritative` — correct (after FixedUpdate)
- `PostUpdate` for `interpolate_transform` — correct (every render frame)

## Custom Test Schedules
- `#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]` on custom struct — correct
- `app.world_mut().run_schedule(TestSchedule)` — valid for running custom schedules in tests

## Observer Trigger Events
- `#[derive(Event)]` (NOT `#[derive(Message)]`) is correct for observer-triggered payloads in Bevy 0.18.1
- `commands.trigger(MyEvent { ... })` is the correct deferred global trigger
- Observer fn signature: `fn handler(trigger: On<MyEvent>, query: Query<...>, mut commands: Commands, mut writer: MessageWriter<M>)` — all SystemParams compose freely
- `#[derive(Event, Clone, Debug)]` on a trigger struct — valid, no extras needed
- Confirmed: `EffectFired` (was `OverclockEffectFired` before refactor/unify-behaviors 2026-03-21) using `#[derive(Event)]` (not Message) is correct for observer dispatch

## Non-mut Query binding with &mut T data
- `armed_query: Query<(Entity, &mut ArmedTriggers)>` without `mut` on the binding is valid when the value is immediately moved into a helper function that declares it `mut`
- `mut` on a binding governs reborrowing in scope, not ownership transfer — moving to a `mut` parameter is always valid
- Confirmed: `bridge_cell_destroyed` and `bridge_bolt_lost` in bridges.rs

## Option<ResMut<T>> for optional system params (re-confirmed)
- `mut active_chains: Option<ResMut<ActiveChains>>` (was `ActiveOverclocks` before refactor/unify-behaviors 2026-03-21) — valid system parameter; None when not inserted
- Pattern used in `bypass_menu_to_playing` in lifecycle/mod.rs — correct
- `mut stats: Option<ResMut<ScenarioStats>>` — same pattern, confirmed correct

## world_mut().despawn(entity) in tests
- `app.world_mut().despawn(entity)` — valid direct World method for despawning in tests (recursive in 0.18)
- Distinct from `commands.entity(e).despawn()` (deferred); world method is immediate

## Multiple MessageWriter params in one system (confirmed 2026-03-20)
- A system may have 2, 3, or more `MessageWriter<T>` params for DIFFERENT message types — valid, no conflict
- `(mut hit_writer: MessageWriter<BoltHitCell>, mut damage_writer: MessageWriter<DamageCell>, mut wall_hit_writer: MessageWriter<BoltHitWall>)` — confirmed correct
- Each `MessageWriter<T>` is an independent `SystemParam`; they don't conflict because they write to different `Messages<T>` resources
- `add_message` is idempotent — registering the same type in multiple plugins (e.g., `BoltHitCell` in `PhysicsPlugin` and in a test app) is safe

## DamageCell message (confirmed 2026-03-20)
- `#[derive(Message, Clone, Debug)]` on `DamageCell` — correct; lives in `cells/messages.rs`
- Registered with `app.add_message::<DamageCell>()` in `CellsPlugin` — correct owner
- Written by `bolt_cell_collision` (physics domain) — cross-domain write is fine since receiver (cells) owns the registration

## BoltHitWall message (confirmed 2026-03-20)
- `#[derive(Message, Clone, Debug)]` on `BoltHitWall` — correct; lives in `physics/messages.rs`
- Registered with `app.add_message::<BoltHitWall>()` in `PhysicsPlugin` alongside the other physics messages — correct
- `MessageReader<BoltHitWall>` in `bridge_wall_impact` — correct consumer pattern

## Query<(Entity, &mut ArmedTriggers)> without mut binding (re-confirmed 2026-03-20)
- Functions `bridge_cell_destroyed` and `bridge_bolt_lost` declare `armed_query: Query<(Entity, &mut ArmedTriggers)>` (no `mut` on the binding)
- Then pass it by value to `evaluate_armed_all(mut armed_query: Query<...>, ...)` which does declare it `mut`
- This is valid: `mut` on a binding only governs reborrow semantics within a scope; moving into a `mut` parameter is always allowed regardless

## System set ordering for bridge systems (confirmed 2026-03-20)
- `.after(BreakerSystems::GradeBump).after(BehaviorSystems::Bridge)` — chaining multiple `.after()` is valid; all constraints are AND-ed
- `.after(PhysicsSystems::BreakerCollision).after(BehaviorSystems::Bridge)` — same pattern, confirmed valid
- Bridge systems ordered after both a message-producer set AND the behaviors bridge set — correct for ensuring messages exist before evaluation

## Patterns That Look Wrong But Are Correct
- `commands.entity(e).despawn()` on UI roots with children — recursive in 0.18+
- `gizmos.circle_2d(vec2, ...)` — Vec2 implements Into<Isometry2d>
- `MessageWriter<AppExit>` — AppExit implements Message
- `(spawn_side_panels, ApplyDeferred, spawn_timer_hud).chain()` — ApplyDeferred works in .chain()
- `commands.entity(panel).with_children(...)` on existing entity — correct
- Cross-plugin ordering with `.after(fn_name)` — correct
- `Has<RequiredToClear>` in query tuple — correct, yields bool
- `world.get_entity(e).is_err()` after `commands.entity(e).despawn()` + tick — valid existence test in 0.18
- `MessageReader<AssetEvent<T>>` — AssetEvent derives Message, not Event; this is correct
- `LayoutChangeContext<'w, 's>` with both lifetimes — correct when struct contains Query/Commands
- `ctx.cell_config.is_changed() && !ctx.cell_config.is_added()` — correct change detection idiom

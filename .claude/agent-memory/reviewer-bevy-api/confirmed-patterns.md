---
name: Confirmed Bevy 0.18.1 API Patterns
description: Patterns verified correct for Bevy 0.18.1 in this project — do not re-flag these
type: reference
---

# Confirmed Correct Patterns for Bevy 0.18.1

## Asset Types
- `#[derive(Asset, TypePath, Deserialize, Clone, Debug)]` — correct derive combo for RON-loadable assets in Bevy 0.18.1; `Asset` requires `TypePath`; `Deserialize` satisfies `DeserializeOwned` bound on `SeedableRegistry::Asset`; consistent across all project asset types (BoltDefinition, CellDefinition, BreakerDefinition, WallDefinition, etc.)
- `app.init_asset::<T>()` — correct registration call for custom asset types in Bevy 0.18.1
- `#[derive(Resource, Debug, Default)]` on a registry struct — correct; `SeedableRegistry` bounds require `Resource + Default + Send + Sync + 'static`

## SeedableRegistry Implementation Pattern (rantzsoft_defaults)
- Trait source: `rantzsoft_defaults/src/registry.rs`
- Required methods: `seed(&mut self, assets: &[(AssetId<Self::Asset>, Self::Asset)])` and `update_single(&mut self, id: AssetId<Self::Asset>, asset: &Self::Asset)`
- `update_all` is provided (default impl: `*self = Self::default(); self.seed(assets)`)
- `AssetId<Self::Asset>` is the correct Bevy 0.18.1 type for asset identity; imported via `bevy::prelude::*`
- `Deserialize` (not `DeserializeOwned`) in the derive is correct — `DeserializeOwned` is a blanket impl for all `T: for<'de> Deserialize<'de>` with no lifetime params; lifetime-free structs satisfy it
- Confirmed in WallRegistry (walls/registry/core.rs) — seed clears then inserts by name, update_single upserts by name, ignoring the `id` arg; this is the correct pattern for name-keyed registries

## Message System
- `#[derive(Message, Clone, Debug)]` — correct derive for Bevy 0.18 message types
- `app.add_message::<T>()` — correct registration call (NOT add_event)
- `MessageWriter<'w, T>` — correct system param for sending messages
- `MessageReader<'w, T>` — correct system param for reading messages
- `Messages<T>` resource — accessed via `app.world().resource::<Messages<T>>()` in tests
- `.iter_current_update_messages()` — correct method on `Messages<T>` to read this frame's messages
- `Messages<T>.write(msg)` — valid direct write method on `Messages<T>` resource (confirmed docs.rs 0.18.1); used in `fire()` World-access functions via `world.resource_mut::<Messages<T>>().write(...)`
- `MessageWriter` is `SystemParam` — two writers for different types in one system are valid
- `type CollisionWriters<'a> = (MessageWriter<'a, A>, MessageWriter<'a, B>)` — valid tuple SystemParam alias

## Query API
- `query.single()` returns `Result` in Bevy 0.15+ — use `let Ok(x) = query.single() else { return; }`
- `Query<BoltCollisionData, ActiveFilter>` — `#[derive(QueryData)]` named struct as query data, filter type alias — both valid (formerly `CollisionQueryBolt` tuple alias; same API fact applies)
- `type WallLookup<'w, 's> = Query<'w, 's, (...), (With<Wall>, Without<Bolt>)>` — valid lifetime-annotated query alias
- `Query<(Has<Cell>, Option<&'static Hp>), Without<Bolt>>` — Has<T> and Option<&T> as query data correct (CellHealth replaced by Hp in unified death pipeline)
- `candidate_lookup.get(hit.entity)` — valid query get by entity

## Component Spawning (post-0.15)
- `Mesh2d(...)`, `MeshMaterial2d(...)` — correct; no *Bundle structs
- `Camera2d` directly (not `Camera2dBundle`) — correct

## Required Components
- `#[require(Spatial2D, InterpolateTransform2D, Velocity2D)]` on Component — correct Bevy 0.15+ API
- `#[require(Spatial2D, CleanupOnNodeExit)]` — correct

## State API
- `#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]` for top-level states — correct
- `#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]` + `#[source(GameState = GameState::Playing)]` — correct Bevy 0.15+ sub-state derive
- `run_if(in_state(PlayingState::Active))` — correct state-gated system pattern
- `OnEnter(GameState::Playing)` — correct schedule for state entry
- `app.init_state::<GameState>()` + `app.add_sub_state::<PlayingState>()` — correct Bevy 0.15+ state registration API; confirmed in StatePlugin and many test harnesses
- `in_state(GameState::TransitionOut).or(in_state(GameState::TransitionIn))` — valid Condition combinator for multi-state run_if
- `OnExit(PlayingState::Paused)` — correct OnExit schedule for sub-states; Bevy fires OnExit for sub-state when parent state or sub-state transitions away
- `Res<State<PlayingState>>` + `.get()` — correct way to read current sub-state value in a system
- `ResMut<NextState<PlayingState>>` + `.set(...)` — correct way to request sub-state transition
- `bevy::state::app::StatesPlugin` in test harnesses — correct plugin import path for headless state tests

## UI Components (Bevy 0.15+ bundle-free)
- `Node { width: Val::Percent(100.0), height: Val::Percent(100.0), position_type: PositionType::Absolute, ..default() }` — correct post-0.15 UI node component (not NodeBundle); spawned in a tuple with other components
- `BackgroundColor(color)` — correct component (not part of a bundle); used alongside Node
- `bg_color.0.with_alpha(alpha)` — correct way to update color alpha on `BackgroundColor` component (`.0` is the inner `Color`)

## SystemParam Derive
- `#[derive(SystemParam)] struct Foo<'w> { writer: MessageWriter<'w, T>, ... }` — correct
- `Result<MessageWriter<'w, T>, SystemParamValidationError>` as a `#[derive(SystemParam)]` field — VALID in Bevy 0.18; allows graceful degradation when message type not registered
- `SystemParamValidationError` from `bevy::ecs::system` — correct import path

## Time API
- `Res<Time>` in `FixedUpdate` — valid; resolves to `Time<Fixed>` semantics automatically
- `Res<Time<Fixed>>` — also valid and explicit
- `time.delta()` / `time.delta_secs()` — correct methods
- In tests: `app.world_mut().resource_mut::<Time<Fixed>>().accumulate_overstep(timestep)` — correct way to drive FixedUpdate in tests

## SystemSet
- `#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]` — correct derive for system sets

## World Direct Access (fire/reverse functions)
- `world.despawn(entity)` — valid in Bevy 0.18.1; called from `fire()`/`reverse()` World-access functions where entities confirmed to exist via prior query
- `world.get_entity_mut(entity)` returns `Result` — correct guard before insert/remove
- `world.get::<C>(entity)` / `world.get_mut::<C>(entity)` — correct direct component access
- `world.query::<T>()` / `world.query_filtered::<T, F>()` — correct for fire/reverse World access functions

## Time API (FixedUpdate systems)
- `Res<Time<Fixed>>` + `.timestep().as_secs_f32()` — correct in FixedUpdate; timestep == delta inside FixedUpdate
- `Res<Time>` + `.delta_secs()` — also correct in FixedUpdate (resolves to Time<Fixed> automatically)
- Both patterns are functionally equivalent inside FixedUpdate; style inconsistency is NOT a bug
- `Res<Time<Fixed>>` + `.timestep()` used for emitter timer accumulation (distinct from expansion dt)

## World Query + get_mut Pattern (speed_boost.rs — confirmed correct)
- `let boosts = world.get::<T>(entity).cloned();` then `let mut query = world.query::<SpatialData>(); query.get_mut(world, entity)` — valid; `.cloned()` releases the immutable borrow before the mutable query borrow starts; `QueryState::get_mut(&mut self, &'w mut World, Entity)` is the correct exclusive World accessor API; confirmed in `speed_boost.rs:47-53`
- `world.query::<SpatialDataMutableType>()` returns an owned `QueryState`; calling `.get_mut(world, entity)` on it is the correct pattern for per-entity exclusive World access in World-access functions

## Screenshot API (Bevy 0.18.1)
- Correct import path: `bevy::render::view::window::screenshot::{Screenshot, save_to_disk}` — NOT `bevy::render::view::screenshot`
- `Screenshot::primary_window()` — correct method; returns a `Screenshot` component; no args
- `save_to_disk(path)` — standalone function returning `impl FnMut(On<'_, '_, ScreenshotCaptured>)`; used as an observer via `commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path))`
- `commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path))` — correct spawning pattern

## Window API (Bevy 0.18.1)
- `WindowResolution::new(width: u32, height: u32)` — takes `u32` physical pixels (confirmed from docs.rs + github source)
- `bevy::window::WindowResolution` — NOT in `bevy::prelude`; must use fully-qualified path
- `WindowPosition::At(IVec2)` — correct; IVec2 holds pixel coordinates
- `WindowPosition` — IS in `bevy::window::prelude` → re-exported through `bevy::prelude::*`; no need for fully-qualified path
- `UiScale` — in `bevy::prelude`, struct with inner `f32`; `ui_scale.0` is the f32 multiplier
- `PrimaryWindow` — correct marker component, in `bevy::window`; used as `With<PrimaryWindow>` query filter
- `query.single()` — returns `Result` in Bevy 0.15+; use `let Ok(x) = query.single()` pattern
- `commands.remove_resource::<R>()` — valid Bevy 0.18 Commands API; `R: Resource`; removes resource from World
- Monitor/PrimaryMonitor API facts (types, fields, import paths, query pattern) — see `../researcher-bevy-api/api-monitor-and-winit.md`

## Run Conditions
- `resource_exists::<T>` — in `bevy::prelude`; used as `system.run_if(resource_exists::<MyResource>)` (no call parens — it IS the condition function item)
- `Option<Res<T>>` as system parameter — valid; system still runs even if resource is absent; `None` when absent

## Other
- `Bloom` from `bevy::post_process::bloom::Bloom` — correct 0.18 path
- `Projection::from(OrthographicProjection { ... })` — correct 0.18 API
- `Local<Vec<T>>` as system param — valid; reuses allocation across frames
- `commands.entity(e).despawn()` — correct for leaf entities (no children to recurse)
- `Has<T>` in query data tuple (not filter) — correct; returns `bool`, confirmed for DamageVisualQuery and breaker queries

## Test App Patterns (feature/missing-unit-tests — confirmed correct)
- `App::new().add_plugins(MinimalPlugins)` — correct minimal test harness for ECS+FixedUpdate tests
- `add_message::<T>()` in test_app() — correct message registration for test harness

## State Plugin / Lifecycle Crate Patterns (feature/effect-placeholder-visuals — confirmed correct)
- `MessageWriter<ChangeState<NodeState>>` as a system param — correct; `ChangeState<S>` is a `#[derive(Message)]` type in rantzsoft_stateflow; `MessageWriter<'w, T>` is the correct Bevy 0.18 param; `.write(ChangeState::new())` is the correct send call
- `ResMut<Time<Virtual>>` + `.unpause()` / `.pause()` — confirmed correct in Bevy 0.18.1 for virtual time control
- `ResMut<NodeOutcome>` + `node_outcome.result = NodeResult::Quit` — correct mutable resource mutation in a system param
- `Route::from(S).to_dynamic(fn)` — project-local typestate builder API (not Bevy core); `fn(&World) -> S` is the correct closure signature; passing a named function (`resolve_node_next_state`) and an inline closure are both valid
- `Route::from(S).to_dynamic(fn).with_transition(T).when(fn)` — correct chaining; typestate enforces no double-set; `.to_dynamic` accepts `impl Fn(&World) -> S + Send + Sync + 'static`
- `app.add_systems(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>)` — correct; `cleanup_on_exit<S>` is a free function system in rantzsoft_stateflow taking `(Commands, Query<Entity, With<CleanupOnExit<S>>>)`; wiring to `OnEnter(State::Teardown)` is the project's cleanup pattern
- `CleanupOnExit::<S>::default()` in spawn tuples — correct; type has `impl Default` via `PhantomData`; used in `commands.entity(e).insert(...)`, `commands.spawn((..., CleanupOnExit::<S>::default(), ...))`, and as `#[require]` fields
- `#[require(Spatial2D, CleanupOnExit<NodeState>)]` on `#[derive(Component)]` structs — correct Bevy 0.15+ required components syntax; generic type parameters inside `#[require(...)]` are supported
- `Messages<ChangeState<NodeState>>` resource accessed via `app.world().resource::<Messages<T>>()` in tests — confirmed correct test pattern for asserting message writes
- `app.world().resource::<Messages<T>>().iter_current_update_messages().count()` — confirmed correct assertion idiom
- Two `ResMut<...>` params for DIFFERENT types in one system (`ResMut<Time<Virtual>>` + `ResMut<NodeOutcome>`) — valid; no world access conflict
- `app.world().resource::<Time<Fixed>>().timestep()` + `app.world_mut().resource_mut::<Time<Fixed>>().accumulate_overstep(timestep)` then `app.update()` — correct pattern to advance one FixedUpdate tick
- `app.world_mut().query::<&T>().iter(app.world()).next().unwrap()` — correct in narrow tests where the entity count is constrained; lint is `clippy::unwrap_used` (warn not deny) so allowed in tests
- `app.world_mut().query_filtered::<&T, With<U>>().iter(app.world())` — correct; mutable borrow for QueryState then immutable for iteration
- `app.add_systems(FixedUpdate, system.after(PhysicsSystems::MaintainQuadtree))` — correct ordering for collision systems that depend on quadtree being populated
- `app.world().get::<T>(entity)` — correct direct component access in test assertions (Bevy 0.18)
- `app.world().get_entity(entity).is_err()` — correct way to check entity despawned in Bevy 0.16+ (returns `Result`)
- `init_resource::<Assets<Mesh>>()` + `init_resource::<Assets<ColorMaterial>>()` in test apps that call spawning systems needing asset handles — correct; avoids panic when system accesses these asset stores
- `add_systems(Startup, spawn_cells_from_layout)` — correct when testing a one-shot spawn system that should only run once on startup
- Entities MUST have `Aabb2D` + `CollisionLayers` + `GlobalPosition2D` to be registered in `CollisionQuadtree`; entities missing these are invisible to collision systems

## Position Source Pattern in fire()/reverse() World Functions
- All World-access fire functions must use `world.get::<Position2D>(entity)` — NOT `world.get::<Transform>(entity)`
- This is the project-wide convention: bolt domain uses Position2D exclusively; Transform is only for rendering
- `chain_lightning/effect.rs` — FIXED in rework: now uses `world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0)`. No Transform fallback. Correct.
- `piercing_beam/effect.rs` — FIXED (feature/full-verification-fixes): `fire()` uses `super::super::entity_position(world, entity)` which is `Position2D -> Vec2::ZERO` only. No Transform fallback. Correct.

## Screenshot API (Bevy 0.18.1)
- `Screenshot::primary_window()` — correct constructor; returns a `Screenshot` component bundle
- `.observe(save_to_disk(path))` — correct; `save_to_disk(PathBuf)` returns an observer system that handles `ScreenshotCaptured`; used via `.observe()` on the spawned entity
- `commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path.clone()))` — confirmed correct full pattern for saving screenshots
- `Screenshot` + `save_to_disk` — imported from `bevy::render::view::screenshot::{Screenshot, save_to_disk}`
- Researched note: async, takes at least 2 frames to complete; `DefaultPlugins` includes the required subsystem automatically

## SystemParam Derive — Two Queries Same Archetype, Different Components
- `#[derive(SystemParam)]` struct with two Query fields both filtered `With<Breaker>`:
  `Query<Entity, With<Breaker>>` (reads Entity — no component, no conflict)
  `Query<&mut BoundEffects, With<Breaker>>` (writes BoundEffects)
- This is valid in Bevy 0.18: queries only conflict when they access the SAME component with conflicting mutability
- Entity access is never a component conflict
- Confirmed correct in `propagate_breaker_changes/system.rs`

## query.single() Return Type (Bevy 0.15+)
- `query.single()` returns `Result<Q, QuerySingleError>` (not the item directly)
- `let Ok(window) = windows.single()` — correct `if let Ok(...)` usage
- `windows.single()` in app.rs:497 in a `sync_ui_scale` system — confirmed correct

## EntropyEngine Component
- `EntropyEngineState` is `pub` (not `pub(crate)`) because tests in same file need it and it's a component — correct
- `OnEnter(PlayingState::Active)` for reset system is correct for sub-state entry scheduling

## Option<&'static mut T> in Query Type Aliases
- `Option<&'static mut ComponentT>` in a `type AliasQuery = (...)` type alias — correct for Bevy 0.18 query tuples; `'static` lifetime required in type aliases
- Same applies to any `Option<&'static T>` or `Option<&'static mut T>` in query data tuples
- Confirmed: `type DamageVisualQuery` in `cells/queries.rs` is correct Bevy 0.18 usage
- NOTE: ShieldActive (formerly used in this query) was ELIMINATED in Shield refactor (2026-04-02)

## Let-Chain Syntax in System Code
- `if let Some(ref mut x) = opt && x.field > 0 { ... }` — valid Rust 2024 edition feature; edition 2024 stabilized `let_chains`; breaker-game uses `edition = "2024"`
- No feature flag needed; this is a stable language feature in edition 2024

## query_filtered in Tests
- `world.query_filtered::<&T, With<U>>()` in unit tests — correct Bevy 0.18 direct World API
- `app.world_mut().query_filtered::<&T, With<U>>()` then `.iter(app.world())` — correct pattern; mutable borrow to create QueryState, then immutable to iterate

## Position Source — shockwave/explode/gravity_well fire() (ALL FIXED)
- `shockwave::fire()`, `explode::fire()`, `gravity_well::fire()`, `piercing_beam::fire()` — ALL FIXED in feature/full-verification-fixes: all now use `super::super::entity_position()` (Position2D → Vec2::ZERO), and spawned entities carry `Position2D`, not Transform
- See the CRITICAL RULE section below for the complete status of all effects

## commands.entity(e).remove::<T>() for Deferred Component Removal
- `commands.entity(e).remove::<T>()` — correct deferred component removal in Bevy 0.18; `Commands::entity().remove()` buffers the removal for end-of-frame application
- This is correct when the system also reads/writes the component in the same frame via a `Query`; the deferred removal doesn't conflict with current frame query access
- NOTE: Original example used `ShieldActive` but that type was ELIMINATED in Shield refactor (2026-04-02)

## insert_if_new with Tuple Bundles
- `commands.entity(entity).insert_if_new((BoundEffects::default(), StagedEffects::default()))` — correct; insert_if_new accepts `impl Bundle`, and tuples of components are Bundles; confirmed Bevy 0.18.1
- `entity_ref.insert_if_new(...)` on `EntityWorldMut` — also valid; same Bundle acceptance

## Manual insert-if-absent via get + insert on EntityWorldMut (commands.rs)
- `if entity_ref.get::<T>().is_none() { entity_ref.insert(T::default()); }` — correct pattern in Bevy 0.18 when you need conditional insert inside a `Command::apply`; `EntityWorldMut::get` returns `Option<&T>`, `EntityWorldMut::insert` accepts `impl Bundle`
- Used in `ensure_effect_components` in `effect/commands.rs` — CORRECT; distinct from `insert_if_new` (same end result, explicit guard approach)

## commands.queue with Closures
- `commands.queue(move |world: &mut World| { ... })` — correct; Commands::queue (renamed from Commands::add in 0.15) accepts closures matching `|&mut World|` as well as types implementing Command
- This is the correct pattern for deferred World-access within a system that also uses Commands

## Option<Res<T>> and Option<ResMut<T>> as SystemParams
- `catalog: Option<Res<ChipCatalog>>` in a system signature — valid Bevy 0.18 SystemParam; returns None when resource is not present
- `mut inventory: Option<ResMut<ChipInventory>>` — valid; same pattern
- These allow graceful degradation when resources may not be registered (e.g., during scenario seeding)

## SystemParam Derive with Query Fields ('w, 's)
- `#[derive(SystemParam)] struct Foo<'w, 's> { q: Query<'w, 's, Entity, With<T>>, ... }` — correct when struct contains Query fields; both lifetimes required for Query
- `#[derive(SystemParam)] struct Foo<'w> { ... }` — correct when struct contains only Res/ResMut/MessageWriter (no Query)
- Both patterns confirmed in this codebase (DispatchTargets, ChainLightningWorld, CellSpawnContext, etc.)

## ApplyDeferred in OnEnter Chains
- `(seed_initial_chips, init_scenario_input, ApplyDeferred, tag_game_entities, ...).chain()` in `OnEnter(GameState::Playing)` — valid Bevy 0.18 pattern; ApplyDeferred flushes deferred commands between steps in a chained system set
- `.after(BoltSystems::InitParams)` on an OnEnter chain — valid scheduling constraint

## Local<bool> Guard Pattern
- `mut done: Local<bool>` in a system + `if *done { return; }` then `*done = true;` — correct Bevy 0.18 one-shot pattern; Local persists across invocations within the app lifetime
- Used in scenario runner for apply_pending_bolt_effects, apply_pending_cell_effects, apply_pending_wall_effects, seed_initial_chips, deferred_debug_setup

## world.entity_mut() vs world.get_entity_mut()
- `world.entity_mut(entity).insert(...)` — panics if entity doesn't exist; used in fire() functions where entity existence is guaranteed by prior query match
- `world.entity_mut(entity).remove::<T>()` — panics if entity doesn't exist; used in reverse() functions; established convention for fire/reverse functions
- `world.get_entity_mut(entity)` returns Result — used in Command::apply implementations (PushBoundEffects, TransferCommand) where entity existence is NOT guaranteed
- The distinction is: World-access fire/reverse functions (confirmed entity exists) use entity_mut(); Command impls use get_entity_mut()

## for _ in reader.read() {} — Message Drain Pattern
- `for _ in reader.read() {}` — valid pattern to drain a MessageReader without processing messages (e.g., when a required resource is absent)

## despawn() is Recursive in Bevy 0.16+
- `commands.entity(e).despawn()` — in Bevy 0.16+, this recursively despawns all children (equivalent to old `despawn_recursive()`)
- `despawn_children()` — despawns children but NOT the parent entity
- `despawn_related::<Children>()` — the 0.16+ way to despawn only children
- No `despawn_recursive()` needed in Bevy 0.18; plain `despawn()` is recursive

## ChildOf and Children in Bevy 0.18
- `ChildOf(parent_entity)` — correct component for establishing parent relationship
- `ChildOf::parent()` — correct method to get the parent Entity from a ChildOf component
- `Children` component — auto-populated when ChildOf is inserted; provides slice iteration via `Deref<Target=[Entity]>`
- `children.iter()` — correct iteration over child entity slice
- `EntityWorldMut::add_child(entity)` — correct API to add a child in 0.18
- `entity_mut(parent).add_child(child)` — correct World-access variant

## Transform vs Position2D in Physics vs Rendering Systems (CRITICAL RULE)
- Bolt entities use `Position2D` (authoritative physics position) + `InterpolateTransform2D` (renders by interpolating to Transform)
- `Transform` on bolt entities is a RENDERED/INTERPOLATED value — one-tick stale relative to physics
- Physics systems (FixedUpdate) MUST query `Position2D` or `GlobalPosition2D` for bolt/cell position
- Rendering/debug systems (Update, gizmos, egui) MAY query `Transform` — they're displaying visual position
- `gravity_well.rs` — FIXED (feature/full-verification-fixes): `apply_gravity_pull` queries `&Position2D` on wells and bolts; `fire()` uses `super::super::entity_position()` (Position2D only, no Transform fallback). All correct.
- `piercing_beam/effect.rs` — FIXED (feature/full-verification-fixes): `fire()` now uses `super::super::entity_position(world, entity)` (Position2D → Vec2::ZERO only). Process system uses `GlobalPosition2D` for cell positions. No Transform involved. Correct.
- `shockwave/effect.rs` — FIXED (feature/full-verification-fixes): `fire()` now uses `super::super::entity_position()` (Position2D); spawned entity carries `Position2D`, not Transform. Correct.
- `explode/effect.rs` — FIXED (feature/full-verification-fixes): `fire()` now uses `super::super::entity_position()` (Position2D); spawned entity carries `Position2D`. Correct.
- `chain_lightning/effect.rs` arc_transforms — CORRECT: ChainLightningArc entities are pure rendering objects; using Transform on them is right
- `pulse/effect.rs` — CORRECT: emitter reads `&Position2D` from emitter entity; ring carries `Position2D`; `apply_pulse_damage` reads `&Position2D` from ring entity. No Transform. Correct.

## Typestate Builder + World Spawn Patterns (feature/chip-evolution-ecosystem — confirmed correct)
- `fn build_core(...) -> impl Bundle + use<>` — valid Rust 2024 edition precise-capturing syntax; `use<>` captures nothing (no lifetime/type params); prevents overcapturing in RPIT; correct in edition 2024
- `world.spawn(core)` returns `EntityWorldMut`; calling `.insert(...)` multiple times on the returned `EntityWorldMut` (without releasing it) is valid; `EntityWorldMut` holds the `&mut World` borrow
- `world.entity_mut(entity).insert(...)` after `world.spawn(bundle)` in a fire() function — entity existence guaranteed by just spawning; panicking `entity_mut` is correct here
- `#[query_data(mutable)]` attribute on `#[derive(QueryData)]` — valid Bevy 0.18.1; generates both mutable (`SpatialData`) and read-only (`SpatialDataReadOnly`) variants; mutable struct uses `&'static mut` fields
- `world.query_filtered::<&Position2D, With<Breaker>>().iter(world).next().map(|p| p.0)` — correct one-liner in exclusive system; QueryState temporary is live for the whole expression; `.map(|p| p.0)` copies `Vec2` (Copy) before borrow ends
- `fn spawn_bolt(world: &mut World)` registered via `add_systems(OnEnter(...), spawn_bolt)` — valid exclusive system registration; Bevy 0.18 implements `IntoSystem` for `fn(&mut World)`
- `world.resource_mut::<Messages<BoltSpawned>>().write(BoltSpawned)` — correct write to message resource from exclusive system; `Messages` is in `bevy::prelude` for 0.18.1
- `Bolt::builder().at_position(...).config(&config).with_velocity(vel).extra().spawn(world)` — correct chained builder + World spawn in fire() functions; returns `Entity`
- `const fn with_lifespan(mut self, duration: f32) -> Self` / `const fn with_radius(mut self, r: f32) -> Self` — valid `const fn` for methods that only assign `f32` into `Option<f32>` fields (Copy types)

## Invariant Checker Query Patterns (feature/scenario-coverage — confirmed correct)
- Two queries with overlapping components but disjoint filters are NOT a conflict: `Query<..., With<ScenarioTagBolt>>` + `Query<..., With<ScenarioTagBreaker>>` both reading `&Aabb2D` — valid in Bevy 0.18; disjoint filters on different tags prevent archetype overlap
- `type BreakerAabbQuery<'w, 's> = Query<'w, 's, (Entity, &'static Aabb2D, &'static BreakerWidth, &'static BreakerHeight, Option<&'static EntityScale>), With<ScenarioTagBreaker>>` — correct Bevy 0.18 lifetime-annotated query alias with static component refs
- `check_size_boost_in_range`: DELETED in cache-removal refactor. NOTE: `ActiveSizeBoosts` and `EffectiveSizeMultiplier` replaced by `EffectStack<SizeBoostConfig>` in effect_v3.
- `check_gravity_well_count_reasonable`: `Query<Entity, With<GravityWellMarker>>` — correct entity-only query with marker filter

## SystemParam Derive with Option<Res> + Option<ResMut> (confirmed in frame_mutations.rs)
- `PauseControl<'w>` with `Option<Res<'w, State<PlayingState>>>` + `Option<ResMut<'w, NextState<PlayingState>>>` — valid `#[derive(SystemParam)]`; no conflict because `State<T>` and `NextState<T>` are distinct resource types
- `MutationTargets<'w, 's>` mixing `Option<ResMut<RunStats>>`, `Option<ResMut<ChipInventory>>`, `Commands`, and `Query` — valid; all distinct types; 'w, 's lifetimes required because of Query field

## apply_gravity_pull Query Analysis (confirmed correct in Bevy 0.18)
- `wells: Query<(&Position2D, &GravityWellConfig), With<GravityWellMarker>>` + `bolts: Query<(&Position2D, &mut Velocity2D), With<Bolt>>` — safe because Bolt and GravityWellMarker are disjoint tags; no entity can have both; Position2D read-only in wells, Position2D read-only in bolts — no write conflict

## resource_exists + .and() Condition Combinator (confirmed for Bevy 0.18)
- `resource_exists::<T>.and(in_state(S::Variant))` — valid; `resource_exists` is a function that implements `Condition`; the `Condition` trait provides `.and()` / `.or()` / `.nand()` / etc. as combinator methods
- This pattern is the correct way to gate a system on BOTH a resource existing AND a state predicate
- Used in `tether_beam/effect.rs` register(): `maintain_tether_chain.run_if(resource_exists::<TetherChainActive>.and(in_state(PlayingState::Active)))` — CORRECT
- `world.remove_resource::<T>()` — confirmed present in Bevy 0.18 World API
- `world.insert_resource(value)` — confirmed present in Bevy 0.18 World API

## Entity::index() Return Type (Bevy 0.18)
- `Entity::index()` returns `EntityIndex` (NOT `u32`) in Bevy 0.18
- `EntityIndex` implements `Ord`, `PartialOrd`, `Eq`, `PartialEq`, `Hash`, `Copy`
- `sort_by_key(|e| e.index())` is valid because `EntityIndex: Ord`
- `Entity::index_u32()` is the companion method returning `u32` directly (equivalent to `self.index().index()`)
- Used in `tether_beam/effect.rs` fire_chain() and maintain_tether_chain() — CORRECT

## world.query_filtered in fire()/reverse() World Functions
- `world.query_filtered::<Entity, With<T>>().iter(world).collect::<Vec<_>>()` — correct pattern for collecting entities matching a filter in a World-access function
- Confirmed in `tether_beam/effect.rs` fire_chain() and reverse() — CORRECT

## spawn_bolts/effect.rs query_filtered Pattern
- `world.query_filtered::<&BoundEffects, (With<Bolt>, Without<ExtraBolt>)>()` — correct compound filter tuple in world.query_filtered; returns QueryState which is then iterated
- `.iter(world).next().cloned()` — correct; QueryState::iter takes &World, then next() and cloned() on Option<&BoundEffects> yield Option<BoundEffects>
- This pattern is safe: query is created (mut borrow), then iterated (immutable borrow) after the exclusive borrow ends via the temporary scope

## Active* Component Query Tuple Size and Method Access (cache-removal refactor — confirmed correct)
- `BoltCollisionData` (formerly `CollisionQueryBolt`) — `#[derive(QueryData)]` named struct with many optional fields — within limits; CORRECT
- `DashQuery` nested tuple with 15 elements in group 1 and 5 in group 2 — outer tuple wraps two inner tuples to avoid exceeding the per-tuple limit; both correct
- `Option<&'static EffectStack<SpeedBoostConfig>>` / `Option<&'static EffectStack<SizeBoostConfig>>` / `Option<&'static EffectStack<DamageBoostConfig>>` as Optional query data — correct Bevy 0.18 pattern
- `.map_or(1.0, EffectStack::<SpeedBoostConfig>::multiplier)` — function reference form; CORRECT Rust
- Same pattern for `EffectStack<SizeBoostConfig>::multiplier`, `EffectStack<DamageBoostConfig>::multiplier`, `EffectStack<PiercingConfig>::total` — all correct
- `EffectStack<T>` is a generic `#[derive(Component)]` with `.multiplier()` / `.total()` / `.aggregate()` methods. NOTE: `Active*` types (`ActiveSpeedBoosts`, `ActiveDamageBoosts`, etc.) were replaced by `EffectStack<T>` in effect_v3 refactor.
- Bevy 0.18 QueryData tuple limit is 15 elements per tuple level; nested tuples each count independently

## SyncBreakerScaleQuery Tuple Type Alias (confirmed correct in Bevy 0.18)
- `type SyncBreakerScaleQuery = (&'static BaseWidth, &'static BaseHeight, &'static mut Scale2D, Option<&'static EffectStack<SizeBoostConfig>>, ...)` — 9-element tuple with `&'static mut` field in a plain `type` alias (not `#[derive(QueryData)]`); valid; mutable refs in `type` aliases require `#[query_data(mutable)]` only when using `#[derive(QueryData)]`; plain `type` aliases can include `&'static mut` directly. NOTE: `ActiveSizeBoosts` replaced by `EffectStack<SizeBoostConfig>`.
- Confirmed used in `Query<SyncBreakerScaleQuery, With<Breaker>>` — the `With<Breaker>` filter is on the outer Query, NOT inside the tuple; correct

## SpatialData with Optional Scale2D / PreviousScale Fields (confirmed correct in Bevy 0.18)
- `pub scale: Option<&'static Scale2D>` and `pub previous_scale: Option<&'static PreviousScale>` as named fields in a `#[derive(QueryData)] #[query_data(mutable)]` struct — both fields are read-only (no `mut`), which is valid inside a mutable QueryData struct; mutable annotation only requires the struct to be decorated, not every field to be mutable
- `PreviousScale` is `#[derive(Component)]` in `rantzsoft_spatial2d`; correct usage in `SpatialData` QueryData

## DispatchInitialEffects Command — world.query_filtered inside Command::apply (confirmed correct)
- Creating `QueryState` from `world.query_filtered::<Entity, F>()` inside `Command::apply(self, world: &mut World)`, calling `.iter(world).collect()`, then dropping the QueryState before calling further World methods — correct; each `query_filtered` call creates and drops an owned QueryState before the next borrow begins; no aliased borrow issue
- Calling `TransferCommand { ... }.apply(world)` directly inside another `Command::apply` — valid; Commands are plain structs implementing `Command`; calling `.apply(world)` directly (instead of queuing) is an immediate synchronous application; correct pattern in Bevy 0.18

## Typestate Builder Pattern (feature/breaker-builder-pattern — confirmed correct)
- `BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Rendered, Primary>` — 7-dimensional typestate builder; terminal `build()` returning `impl Bundle` is correct; `spawn(&mut commands)` calling `commands.spawn(self.build()).id()` and then `commands.stamp_effect(...)` is the correct Commands-based spawn pattern. NOTE: `dispatch_initial_effects` was removed in effect_v3 refactor.
- `BoltBuilder<...> spawn(self, world: &mut World)` — exclusive World-access spawn; `world.spawn(core)` + `.insert(...)` chain on `EntityWorldMut` is confirmed correct pattern (see earlier typestate builder notes)
- `fn build(self) -> impl Bundle` (no `use<>`) — correct when `self` is consumed by value with no borrowed lifetime fields; compiler infers no overcapture; `use<>` optional in edition 2024
- `fn build_core(...) -> impl Bundle + use<>` — also correct for free functions with no type/lifetime params to capture
- `const fn with_lives(mut self, lives: Option<u32>) -> Self` in generic `BreakerBuilder<D,Mv,Da,Sp,Bm,V,R>` — valid `const fn` only when all type parameters are `Copy`/const-compatible at the call site; the match on `Option<u32>` is fine; however, `BreakerBuilder` contains `D`, `Mv`, etc. which are NOT constrained to `Copy` — this const fn only compiles when the compiler determines at monomorphization that the move is valid; NO ISSUE for the current use (typestate markers are all `()` effectively)
- `world.remove_resource::<Assets<Mesh>>().unwrap_or_default()` in exclusive system for borrow-split — correct pattern; avoids `&mut World` aliasing when builder's `rendered()` needs `&mut Assets<Mesh>`; `world.insert_resource(meshes)` re-inserts after spawn; confirmed in `spawn_bolt/system.rs`
- `BoltRadius` is a type alias `pub type BoltRadius = crate::shared::size::BaseRadius` — not a separate component; using it in QueryData fields queries the same component as `BaseRadius`

## commands.spawn() + .id() (confirmed correct in Bevy 0.18)
- `commands.spawn(bundle).id()` — `Commands::spawn` returns `EntityCommands`; `.id()` on `EntityCommands` returns `Entity`; method chaining `.spawn(...).id()` is valid and returns the spawned entity's `Entity` id; no deferred lookup or world access needed
- Used in `breaker/builder/core.rs` spawn() methods — CORRECT

## impl Bundle return types (confirmed correct — plain vs use<>)
- `fn build_core(...) -> impl Bundle + use<>` (no captured lifetimes/type params) — correct; `use<>` captures nothing; prevents overcapturing in RPIT in edition 2024 (already in confirmed-patterns.md)
- `pub fn build(self) -> impl Bundle` (consuming self, no borrows) — also CORRECT in edition 2024; `build()` takes `self` by value so there are no lifetimes to overcapture; `use<>` is optional here; the compiler infers no lifetime dependencies; no error expected
- Both forms coexist correctly in the same file

## EffectCommandsExt methods (confirmed correct — effect_v3)
- `EffectCommandsExt` extension trait on `Commands<'_, '_>`, defined in `effect_v3/commands/ext.rs`. Methods: `fire_effect`, `reverse_effect`, `route_effect`, `stamp_effect`, `stage_effect`, `remove_effect`, `remove_staged_effect`, `track_armed_fire`. NOTE: `dispatch_initial_effects` and `push_bound_effects` were removed in effect_v3 refactor.

## ColorMaterial::from_color + meshes.add / materials.add (Bevy 0.18)
- `meshes.add(Rectangle::new(1.0, 1.0))` — `Assets<Mesh>::add` accepts `impl Into<A>` where `Rectangle: Into<Mesh>`; CORRECT
- `materials.add(ColorMaterial::from_color(color))` — `ColorMaterial::from_color` takes `impl Into<Color>`; accepts `Color` directly; `Assets<ColorMaterial>::add` is correct; pattern confirmed across multiple files in this project

## #[serde(deny_unknown_fields)] on Asset structs (confirmed correct)
- `#[serde(deny_unknown_fields)]` on a `#[derive(Asset, TypePath, Deserialize, Clone, Debug)]` struct — valid; serde attribute applies to `Deserialize`; `Asset` derive is independent; no conflict; RON deserialization will reject unknown fields at runtime which is the intended behavior
- Used on `BreakerDefinition` — CORRECT; tests confirm correct RON round-trips

## CollisionLayers::new(membership, mask) — project-local crate
- `CollisionLayers::new(BREAKER_LAYER, BOLT_LAYER)` — `CollisionLayers` is from `rantzsoft_physics2d`, NOT bevy; `new(membership: u32, mask: u32) -> Self`; `BREAKER_LAYER` and `BOLT_LAYER` are `u32` constants; call is CORRECT
- `CollisionLayers::new(WALL_LAYER, BOLT_LAYER)` — same API, wall-specific constants; CORRECT
- `Aabb2D::new(center: Vec2, half_extents: Vec2)` — also from `rantzsoft_physics2d`; `Aabb2D::new(Vec2::ZERO, Vec2::new(w/2, h/2))` is CORRECT

## Wall Builder Core (Wave 2 — confirmed correct)
- `fn build(self) -> impl Bundle + use<S>` where `S` is a type param — valid Rust 2024 RPIT; `use<>` captures listed type params; `use<S>` captures `S` (a type parameter, not a lifetime); CORRECT
- `fn build_core(position: Vec2, half_extents: Vec2) -> impl Bundle + use<>` — no type/lifetime params; `use<>` captures nothing; CORRECT (already known pattern, confirmed again in wall builder)
- Nested sub-tuple bundle: `((Wall, Position2D, Scale2D), (Aabb2D, CollisionLayers, GameDrawLayer))` returned as `impl Bundle` — outer 2-tuple where each arm is a 3-tuple; all satisfy `Bundle`; CORRECT
- `RootNode::Stamp(target, tree)` — `RootNode` has variants `Stamp(StampTarget, Tree)` and `Spawn(EntityKind, Tree)`; tuple variant pattern matching; CORRECT. NOTE: `RootEffect::On` was replaced by `RootNode::Stamp` in effect_v3 refactor.
- `commands.stamp_effect(entity, source, tree)` — method from `EffectCommandsExt` extension trait on `Commands<'_, '_>` (defined in `effect_v3/commands/ext.rs`); signature `fn stamp_effect(&mut self, entity: Entity, source: String, tree: Tree)`; CORRECT; used by bolt, cell, chip, and wall domains. NOTE: `push_bound_effects` was removed in the effect_v3 refactor.
- `Wall` component has `#[require(Spatial2D, CleanupOnNodeExit)]` — confirmed in `walls/components.rs`; covered by existing confirmed-patterns entry for `#[require]`
- `Mesh2d(handle)` and `MeshMaterial2d(handle)` — post-0.15 component-based rendering; confirmed correct (previously established)

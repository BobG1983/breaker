---
name: Confirmed Bevy 0.18.1 API Patterns
description: Patterns verified correct for Bevy 0.18.1 in this project — do not re-flag these
type: reference
---

# Confirmed Correct Patterns for Bevy 0.18.1

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
- `Query<CollisionQueryBolt, ActiveFilter>` — type alias as query data, filter type alias — both valid
- `type WallLookup<'w, 's> = Query<'w, 's, (...), (With<Wall>, Without<Bolt>)>` — valid lifetime-annotated query alias
- `Query<(Has<Cell>, Option<&'static CellHealth>), Without<Bolt>>` — Has<T> and Option<&T> as query data correct
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

## Other
- `Bloom` from `bevy::post_process::bloom::Bloom` — correct 0.18 path
- `Projection::from(OrthographicProjection { ... })` — correct 0.18 API
- `Local<Vec<T>>` as system param — valid; reuses allocation across frames
- `commands.entity(e).despawn()` — correct for leaf entities (no children to recurse)
- `Has<T>` in query data tuple (not filter) — correct; returns `bool`, confirmed for DamageVisualQuery and breaker queries

## Test App Patterns (feature/missing-unit-tests — confirmed correct)
- `App::new().add_plugins(MinimalPlugins)` — correct minimal test harness for ECS+FixedUpdate tests
- `add_message::<T>()` in test_app() — correct message registration for test harness
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

## EntropyEngine Component
- `EntropyEngineState` is `pub` (not `pub(crate)`) because tests in same file need it and it's a component — correct
- `OnEnter(PlayingState::Active)` for reset system is correct for sub-state entry scheduling

## Option<&'static mut T> in Query Type Aliases
- `Option<&'static mut ShieldActive>` in a `type DamageVisualQuery = (...)` alias — correct for Bevy 0.18 query tuples; `'static` lifetime required in type aliases
- Same applies to any `Option<&'static T>` or `Option<&'static mut T>` in query data tuples
- Confirmed: `type DamageVisualQuery` in `cells/queries.rs` is correct Bevy 0.18 usage

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
- `commands.entity(cell).remove::<ShieldActive>()` — correct deferred component removal in Bevy 0.18; `Commands::entity().remove()` buffers the removal for end-of-frame application
- This is correct when the system also reads/writes the component in the same frame via a `Query`; the deferred removal doesn't conflict with current frame query access

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

## Invariant Checker Query Patterns (feature/scenario-coverage — confirmed correct)
- Two queries with overlapping components but disjoint filters are NOT a conflict: `Query<..., With<ScenarioTagBolt>>` + `Query<..., With<ScenarioTagBreaker>>` both reading `&Aabb2D` — valid in Bevy 0.18; disjoint filters on different tags prevent archetype overlap
- `type BreakerAabbQuery<'w, 's> = Query<'w, 's, (Entity, &'static Aabb2D, &'static BreakerWidth, &'static BreakerHeight, Option<&'static EntityScale>), With<ScenarioTagBreaker>>` — correct Bevy 0.18 lifetime-annotated query alias with static component refs
- `check_size_boost_in_range`: `Query<(Entity, &ActiveSizeBoosts, &EffectiveSizeMultiplier)>` — both immutable, no filter conflict; correct
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

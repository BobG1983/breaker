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

## Position Source Pattern in fire()/reverse() World Functions
- All World-access fire functions must use `world.get::<Position2D>(entity)` — NOT `world.get::<Transform>(entity)`
- This is the project-wide convention: bolt domain uses Position2D exclusively; Transform is only for rendering
- `chain_lightning/effect.rs` — FIXED in rework: now uses `world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0)`. No Transform fallback. Correct.
- `piercing_beam.rs` — STILL has `Position2D -> Transform fallback -> Vec2::ZERO` chain. The Transform fallback is wrong — should be `Position2D -> Vec2::ZERO` only. Still open as of feature/runtime-effects.

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

## Position Source — shockwave/explode fire() Exception
- `shockwave::fire()` and `explode::fire()` use `world.get::<Transform>(entity)` — this is intentional; these effects spawn entities carrying a `Transform` for rendering-integrated position tracking (the shockwave/explode entities are rendering objects, not physics entities)
- The project convention (Position2D not Transform) applies to bolt/cell physics objects, not to standalone effect-spawn entities that carry Transform for their own spatial representation
- Only `chain_lightning.rs` and `piercing_beam.rs` are confirmed wrong (use Transform on the source bolt entity)

## commands.entity(e).remove::<T>() for Deferred Component Removal
- `commands.entity(cell).remove::<ShieldActive>()` — correct deferred component removal in Bevy 0.18; `Commands::entity().remove()` buffers the removal for end-of-frame application
- This is correct when the system also reads/writes the component in the same frame via a `Query`; the deferred removal doesn't conflict with current frame query access

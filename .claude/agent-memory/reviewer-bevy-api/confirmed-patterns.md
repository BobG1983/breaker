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

## Other
- `Bloom` from `bevy::post_process::bloom::Bloom` — correct 0.18 path
- `Projection::from(OrthographicProjection { ... })` — correct 0.18 API
- `Local<Vec<T>>` as system param — valid; reuses allocation across frames
- `commands.entity(e).despawn()` — correct for leaf entities (no children to recurse)

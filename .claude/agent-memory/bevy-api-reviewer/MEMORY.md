# bevy-api-reviewer Memory

## Bevy Version
Bevy 0.18.1, `features = ["2d"]`, `default-features = false`

## Confirmed Correct Patterns for This Version

### Hierarchy
- `ChildOf` (not `Parent`) is the parent component — confirmed correct for 0.18.1
- `child_of.parent()` method is correct
- `commands.entity(e).with_child(bundle)` auto-inserts `ChildOf`

### Despawn
- `commands.entity(e).despawn()` is RECURSIVE in 0.18 (changed from 0.16 migration; `despawn_recursive` was removed)
- The cleanup_entities pattern using `despawn()` is correct even for UI hierarchies

### Query API
- `query.single()` returns `Result<D, QuerySingleError>` in 0.18.1 — fallible
- The `let Ok(...) = query.single() else { return; }` pattern throughout is correct
- `query.single_mut()` also returns a Result

### Messages
- `#[derive(Message)]` + `MessageWriter<T>` + `MessageReader<T>` + `app.add_message::<M>()` — correct
- `AppExit` implements `Message` in 0.18.1 — `MessageWriter<AppExit>` is valid
- `KeyboardInput` is a `Message` (not Event) — `MessageReader<KeyboardInput>` is correct
- `world.write_message(msg)` is how to send messages in tests

### Spawn Patterns
- `Mesh2d(meshes.add(...))` + `MeshMaterial2d(materials.add(...))` — correct (no bundles)
- `Camera2d` as zero-size marker component with required components auto-inserted — correct
- `Sprite::from_color()` / `ColorMaterial::from_color()` — correct
- `Circle::new(r)` + `Rectangle::new(w, h)` in `bevy::prelude` — correct

### UI (Bevy 0.18 - no bundles)
- `Node { ... }` component directly (not `NodeBundle`) — correct
- `Text::new("...")` for UI text — correct
- `Text2d::new("...")` for 2D world text — correct
- `TextFont { font_size, ..default() }` — correct
- `TextFont::from_font_size(f32)` constructor — confirmed valid
- `TextColor(Color::...)` — correct
- `BackgroundColor(Color::...)`, `BorderColor::all(...)` — correct
- `Button` as marker component (no `ButtonBundle`) — correct

### Gizmos API
- `gizmos.circle_2d(impl Into<Isometry2d>, radius, color)` — Vec2 implements Into<Isometry2d> via From<Vec2>
- `gizmos.rect_2d(impl Into<Isometry2d>, Vec2, color)` — same, Vec2 arg is valid
- `gizmos.arrow_2d(Vec2, Vec2, color)` — takes Vec2 directly (not Isometry2d)

### State API
- `#[derive(States)]`, `app.init_state::<S>()`, `in_state(S::Variant)` — correct
- `#[derive(SubStates)]` with `#[source(ParentState = ParentState::Variant)]` — correct
- `OnEnter(S::V)`, `OnExit(S::V)` schedule labels — correct

### EguiPlugin
- `app.add_plugins(EguiPlugin::default())` — correct for bevy_egui 0.39
- Debug UI systems go in `bevy_egui::EguiPrimaryContextPass` schedule — correct

### Fixed Update Testing
- `accumulate_overstep(timestep)` triggers FixedUpdate in tests — correct (NOT advance_by)
- `Time<Fixed>` with `accumulate_overstep` pattern throughout is correct

### Easing
- `EaseFunction::QuadraticIn` etc. in `bevy::math::curve::easing` — correct
- `.sample_clamped(t)` on `EaseFunction` — correct (implements `Curve<f32>`)
- Import: `use bevy::math::curve::{Curve, easing::EaseFunction}` — correct

### Input
- `Res<ButtonInput<KeyCode>>` with `.pressed()`, `.just_pressed()` — correct
- `InputSystems` (plural) system set — correct

### Camera
- `Projection::from(OrthographicProjection { ... })` — correct
- `OrthographicProjection::default_2d()` — correct
- `ScalingMode::AutoMin { min_width, min_height }` — correct
- `Tonemapping::AcesFitted` — safe variant (no LUT required) — correct

### Window
- `window.set_maximized(true)` — correct (no `WindowMode::Maximized`)

### AssetPlugin
- `bevy::asset::AssetPlugin { file_path: "assets".into(), ..default() }` — correct for tests

### bevy_common_assets 0.15 — RonAssetPlugin
- `RonAssetPlugin::<T>::new(&[...])` accepts multiple extensions for one type — CONFIRMED
- `RonAssetPlugin::<UpgradeDefinition>::new(&["amp.ron", "augment.ron", "overclock.ron"])` is valid; one plugin instance handles all three extensions for the same asset type

### bevy_asset_loader 0.25 — Directory Collection
- `#[asset(path = "amps", collection(typed))]` on `Vec<Handle<T>>` — correct for loading all assets in folder as typed handles
- Pattern already used for `cells`, `nodes`, `archetypes` in this codebase — confirmed working
- Same pattern applied to `amps`, `augments`, `overclocks` for `Vec<Handle<UpgradeDefinition>>` is correct

### MessageWriter / MessageReader (confirmed from docs.rs)
- `writer.write(msg)` — returns `MessageId<E>`; this is the correct method name (NOT `send`)
- `reader.read()` — returns `MessageIterator` yielding `&'a M`; used with `for msg in reader.read()` pattern — CORRECT
- `reader.read_with_id()` — yields `(&'a M, MessageId<M>)` pairs if id needed

### Asset Derive Pattern (for RON assets without GameConfig)
- `#[derive(Asset, TypePath, Deserialize, Clone, Debug)]` — correct for plain data assets
- `app.init_asset::<T>()` in tests — correct way to register an asset type without RonAssetPlugin

## Deprecated Patterns Found
(none found in this codebase)

### Schedule Ordering
- `.after(fn_from_another_plugin)` for cross-plugin system ordering — CORRECT, function pointers implement IntoSystemSet via blanket impl
- `.before(fn)` / `.after(fn)` work across plugin boundaries with no restriction

### ApplyDeferred
- `ApplyDeferred` exists in `bevy::ecs::schedule` — CORRECT import path is `bevy::ecs::schedule::ApplyDeferred`
- Using `ApplyDeferred` as an element in a `.chain()` tuple IS effective — the warning "does nothing if called manually or wrapped in a PipeSystem" refers to `.pipe()` (PipeSystem, data-passing composition), NOT to `.chain()` (schedule ordering)
- `.chain()` is `IntoSystemConfigs::chain()` — produces ordered schedule entries; ApplyDeferred placed there will be invoked by the executor at that point
- `(system_a, ApplyDeferred, system_b).chain()` correctly flushes deferred commands (Commands etc.) between system_a and system_b

### Node Fields
- `Node::row_gap: Val` — CONFIRMED exists, type is `Val` (e.g., `row_gap: Val::Px(12.0)`)
- `Node::column_gap: Val` — also exists; `Val::Auto` is invalid for gap fields, treated as zero

### EntityCommands
- `commands.entity(e).insert_if_new(bundle)` — CONFIRMED exists in 0.18.1; inserts bundle without overwriting existing components (leave-old semantics)
- Tuple bundles work: `insert_if_new((ComponentA(v), ComponentB(v)))` — correct

### f32::midpoint
- `f32::midpoint(a, b)` — CONFIRMED stable since Rust 1.85.0 (released 2025-02-20); project uses edition = "2024" which requires 1.85.0+, so always available

## Patterns That Look Wrong But Are Correct
- `commands.entity(e).despawn()` on UI roots with children — CORRECT, despawn() is recursive in 0.18+
- `gizmos.circle_2d(vec2, radius, color)` — CORRECT, Vec2 implements Into<Isometry2d>
- `MessageWriter<AppExit>` — CORRECT, AppExit implements Message in 0.18.1
- `(spawn_side_panels, ApplyDeferred, spawn_timer_hud).chain()` in OnEnter — CORRECT, ApplyDeferred flushes commands between chained systems; the "does nothing" warning only applies to .pipe()
- `commands.entity(panel).with_children(|parent| { ... })` on an existing entity — CORRECT API for adding children to an already-spawned entity in 0.18
- `spawn_lives_display.after(spawn_timer_hud)` where spawn_timer_hud is from ui plugin — CORRECT cross-plugin ordering

## Reference
- bevy-api-expert memory: `.claude/agent-memory/bevy-api-expert/` — already-verified facts; check here before looking up docs.rs
- despawn recursive change: Bevy 0.15→0.16 migration — despawn() is now recursive
- Gizmos: docs.rs/bevy/0.18.1/bevy/gizmos/gizmos/struct.Gizmos.html
- Full review of fix/review-findings branch: `review-fix-review-findings-2026-03-16.md`

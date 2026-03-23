## Stable
- [project-setup.md](project-setup.md) — Feature flags, fast compile, dynamic_linking pattern
- [mesh_input_plugins.md](mesh_input_plugins.md) — MeshPlugin and InputPlugin: paths, what they register, init_asset vs init_resource
- [core-api.md](core-api.md) — Spawn patterns, messages, states, UI, camera, assets, queries
- [easing_api.md](easing_api.md) — EaseFunction variants, Curve trait, EasingCurve usage
- [fixed_update_testing.md](fixed_update_testing.md) — accumulate_overstep pattern, NOT advance_by
- [headless_app.md](headless_app.md) — Headless App, ScheduleRunnerPlugin, AppExit, LogPlugin
- [hierarchy.md](hierarchy.md) — ChildOf (not Parent), with_child, ChildSpawnerCommands
- [keyboard_input.md](keyboard_input.md) — KeyboardInput is Message (not Event), InputSystems set
- [observers_and_oneshot.md](observers_and_oneshot.md) — Observers, triggers, one-shot systems
- [text_input.md](text_input.md) — Text input via KeyboardInput.text, repeat handling, focus, bevy_simple_text_input 0.14.1
- [third_party_crates.md](third_party_crates.md) — bevy_egui, bevy_common_assets, bevy_asset_loader, iyes_progress
- [transform_interpolation.md](transform_interpolation.md) — No built-in support; manual or crate; overstep_fraction() for lerp alpha
- [app_run_vs_update_loop.md](app_run_vs_update_loop.md) — App::run() self-replacement, WinitPlugin runner mechanics, why manual update() cannot drive windowed apps
- [rot2_and_math.md](rot2_and_math.md) — Rot2: constructors, accessors, slerp/nlerp (no lerp), IDENTITY, Default, Reflect
- [transform_systems_set.md](transform_systems_set.md) — TransformSystems::Propagate (plural, NOT TransformSystem::TransformPropagate)
- [require_attribute.md](require_attribute.md) — #[require(...)] syntax: plain, constructor, named-field, expression forms

- [spatial2d_builtin_audit.md](spatial2d_builtin_audit.md) — What Bevy 0.18.1 provides natively vs what must be custom for a spatial2d crate (Position2D, interpolation, z-order, absolute positioning, Transform2D)
- [change_removal_detection.md](change_removal_detection.md) — Added<T>, Changed<T>, Ref<T>, RemovedComponents<T>: signatures, iteration method (.read()), gotchas
- [resource_and_system_ordering.md](resource_and_system_ordering.md) — Resource trait bounds, generic Resource derive, .before()/.after()/.in_set(), cross-plugin set ordering

## Session History
See [ephemeral/](ephemeral/) — not committed.

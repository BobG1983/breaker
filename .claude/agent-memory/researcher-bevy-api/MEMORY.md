## Bevy 0.18.1 API Reference

- [Time API](api-time-and-virtual.md) — Time<Virtual>/Real/Fixed, dilation, pause/unpause, ramp systems
- [Message System](api-message-system.md) — Message derive, MessageWriter/Reader, test injection, on_message, AppExit
- [QueryData Derive](api-query-data.md) — custom named query structs, mutable, nested, Has/With distinction
- [State System](api-states.md) — States/SubStates/ComputedStates, StateTransitionEvent, in_state/state_changed, condition_changed, configure_sets per-schedule
- [World Access and Bundle](api-world-and-bundle.md) — one-shot systems, Commands::run_system, Bundle/BundleInfo introspection
- [World::commands() and flush()](api-world-commands-flush.md) — World::commands() returns Commands on internal queue; World::flush() applies it; no CommandQueue needed in tests
- [Run Conditions](api-run-conditions.md) — combinators (.and/.or/.nand/.nor), resource change detection, Observers limitation
- [UI and Rendering](api-ui-and-rendering.md) — GlobalZIndex overlays, Val variants, UiScale, TextFont, Screenshot API
- [FullscreenMaterial API](rendering-fullscreen-material.md) — FullscreenMaterial trait, ViewTarget ping-pong, PostProcessWrite, Node2d graph anchors

- [Monitor and winit 0.30 API](api-monitor-and-winit.md) — No pre-run monitor query in winit 0.30; Monitor is a Component in Bevy; CoreGraphics blocked by unsafe_code=deny; use child self-tiling pattern

## Session History
See [ephemeral/](ephemeral/) — not committed.

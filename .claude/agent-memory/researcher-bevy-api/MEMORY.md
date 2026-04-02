- [Confirmed Bevy 0.18.1 API Patterns](confirmed-patterns.md) — verified-correct patterns for Time<Virtual>/Real/Fixed, time dilation, ramp systems; QueryData derive; Bundle trait and BundleInfo; Message system; States/SubStates/ComputedStates trait signatures, in_state condition, configure_sets is per-schedule; GlobalZIndex full-screen overlay pattern; StateTransitionEvent fields; Bevy 0.18 breaking change: next_state.set() always triggers OnEnter/OnExit; one-shot systems (register_system/Commands::run_system); on_message run condition with independent cursor; StateTransition schedule placement (after PreUpdate); NextState single-slot semantics
- [FullscreenMaterial API](rendering-fullscreen-material.md) — FullscreenMaterial trait, FullscreenMaterialPlugin, ViewTarget ping-pong, Node2d Core2d graph, blend state limitation (hardcoded None, no specialize()), HDR support

## Session History
See [ephemeral/](ephemeral/) — not committed.

# Condition Evaluation Overview

Conditions are continuous states that can be true or false at any moment. During nodes watch conditions and fire their scoped effects when the condition becomes true, reverse them when it becomes false.

## How it differs from triggers

Triggers are one-time events. A bridge system detects the event, calls `walk_effects`, and the walker matches When/Once/Until nodes. Triggers fire once and are done.

Conditions are polled every frame. A system checks whether each condition is true, compares against the stored previous state, and fires/reverses on transitions. No `walk_effects` call — the condition system directly calls `fire_effect`/`reverse_effect` on the scoped effects.

## Runtime state

`BoundEffects` is a plain `Vec<(String, Tree)>`. Condition state is NOT stored per-entry in BoundEffects. Instead, a separate `DuringActive(pub HashSet<String>)` component on the same entity tracks which During source strings are currently active (condition true). The evaluate_conditions system reads `DuringActive` to detect transitions:
- Source absent from `DuringActive` — condition is currently false (or not yet evaluated)
- Source present in `DuringActive` — condition was true last frame

On transition false→true: insert the source into `DuringActive`, then fire all scoped effects under that During entry.
On transition true→false: remove the source from `DuringActive`, then reverse all scoped effects under that During entry.

## Evaluation order

The `evaluate_conditions` system runs in `EffectV3Systems::Conditions`, after `EffectV3Systems::Tick`. This ensures conditions like ShieldActive reflect shields spawned or despawned by tick systems this frame.

# Condition Evaluation Overview

Conditions are continuous states that can be true or false at any moment. During nodes watch conditions and fire their scoped effects when the condition becomes true, reverse them when it becomes false.

## How it differs from triggers

Triggers are one-time events. A bridge system detects the event, calls `walk_effects`, and the walker matches When/Once/Until nodes. Triggers fire once and are done.

Conditions are polled every frame. A system checks whether each condition is true, compares against the stored previous state, and fires/reverses on transitions. No `walk_effects` call — the condition system directly calls `fire_effect`/`reverse_effect` on the scoped effects.

## Runtime state

Each During entry in BoundEffects carries a `condition_active: Option<bool>` field:
- `None` — not a During entry (When, Once, etc.)
- `Some(false)` — condition was false last frame (initial state on install)
- `Some(true)` — condition was true last frame

This field is runtime-only. It does not appear in RON — it defaults to `None` during deserialization and is set to `Some(false)` when a During entry is installed into BoundEffects.

## Evaluation order

The `evaluate_conditions` system runs in `EffectSystems::Conditions`, after `EffectSystems::Tick`. This ensures conditions like ShieldActive reflect shields spawned or despawned by tick systems this frame.

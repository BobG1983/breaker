# ComboStreak — Removed

`ComboStreak` is NOT a resource in the effect_v3 domain. This file is retained as a pointer.

Combo tracking lives in the run domain resource `HighlightTracker`, field `consecutive_perfect_bumps: u32`.

The `is_combo_active(world, threshold)` condition evaluator in `src/effect_v3/conditions/combo_active.rs` reads `HighlightTracker.consecutive_perfect_bumps` directly — no effect-domain resource is needed.

`HighlightTracker` is defined in `src/run/` and managed by the run domain. The effect system has read-only access to it via world queries.

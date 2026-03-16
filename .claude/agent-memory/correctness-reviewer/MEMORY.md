# correctness-reviewer Memory

## Known Correct Patterns (Do Not Flag)
- `bolt_lost` immediately respawns the bolt with straight-up velocity — intentional per design ("losing position is penalty enough")
- `set_active_layout` wraps `node_index % registry.layouts.len()` — deliberate, not a bug
- `handle_main_menu_input` reads `ButtonInput<KeyCode>` directly rather than `InputActions` — intentional; InputActions is cleared in FixedPostUpdate which is between PreUpdate and Update
- `spawn_bolt` adds `BoltServing` only on first node; subsequent nodes launch immediately — correct and tested
- `animate_bump_visual` subtracts the previous frame's offset before applying the new one — correct differential approach
- `track_node_completion` uses `remaining.is_changed()` — correct guard to avoid spurious `NodeCleared` on frames with no destroyed cells
- `handle_cell_hit` despawns via commands while iterating `reader.read()` — safe; despawn only takes effect when commands flush, not mid-iteration

## Recurring Bug Categories
- **Partial message drain**: `bridge_bolt_lost` uses `reader.read().next().is_none()` which only checks the first message. Multiple simultaneous BoltLost messages (future Prism archetype with multiple bolts) will have extras silently consumed without firing consequences. Harmless with one bolt.

## State Machine Rules
- Valid transitions: Loading→MainMenu, MainMenu→Playing, Playing→NodeTransition→Playing (node advance), Playing→RunEnd (win/timer expire)
- `advance_node` runs `OnEnter(NodeTransition)` and immediately sets `NextState(Playing)` — 1-frame intermediate pattern
- `reset_run_state` runs `OnExit(MainMenu)` — resets node_index and outcome
- `handle_timer_expired` guards on `RunOutcome::InProgress` — prevents timer from overriding a Won run

## ECS Pitfalls Found
- `bridge_bolt_lost` partial drain (see Recurring Bug Categories)
- `apply_bump_velocity` collects messages into a Vec before querying — correct pattern to avoid borrow conflicts between MessageReader and mutable Query

## Math/Physics Notes
- `enforce_min_angle` uses `atan2(|y|, |x|)` — result is always [0, π/2], correct for angle-from-horizontal
- `reflect_top_hit`: `hit_fraction * max_angle + tilt_angle` clamped to `[-max_angle, max_angle]` — tilt can be fully cancelled by clamp when it pushes past the window; this is a design choice
- CCD `remaining -= advance` (not `advance + CCD_EPSILON`) — intentional; prevents sticking at contact surfaces

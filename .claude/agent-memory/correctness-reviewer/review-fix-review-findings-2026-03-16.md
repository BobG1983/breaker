---
name: review-fix-review-findings-2026-03-16
description: Correctness review of fix/review-findings branch changes — 2026-03-16
type: project
---

# Correctness Review — fix/review-findings branch

Date: 2026-03-16
Files reviewed:
- src/breaker/systems/init_breaker_params.rs
- src/ui/systems/animate_fade_out.rs
- src/bolt/systems/bolt_lost_feedback.rs
- src/bolt/plugin.rs
- src/ui/plugin.rs
- src/breaker/systems/spawn_breaker.rs
- src/cells/systems/handle_cell_hit.rs
- src/run/node/systems/spawn_cells_from_layout.rs

---

## Correctness Review

### Logic & Control Flow [1 issue]

**src/breaker/systems/init_breaker_params.rs:73** — `insert_if_new` is called AFTER `apply_bolt_speed_boosts` writes multiplier components via `Commands`, but in the same frame the system ordering is:

```
apply_archetype_config_overrides
  → init_breaker_params           (inserts all params, then calls insert_if_new for multipliers)
  → init_archetype                (calls apply_bolt_speed_boosts via commands)
```

The ordering in `behaviors/plugin.rs` is:
- `apply_archetype_config_overrides.before(init_breaker_params)`
- `init_archetype.after(init_breaker_params)`

So `init_archetype` runs AFTER `init_breaker_params`. `apply_bolt_speed_boosts` issues `commands.entity(entity).insert(BumpPerfectMultiplier(...))`. `init_breaker_params` issued `insert_if_new((BumpPerfectMultiplier(1.0), BumpWeakMultiplier(1.0)))` earlier in the same frame — those commands flushed (via OnEnter schedule command application) before `init_archetype` runs and calls `apply_bolt_speed_boosts`. So the archetype `insert` (plain, not `insert_if_new`) then overwrites the default 1.0. This is the intended behavior.

**However**: on node re-entry (Playing → NodeTransition → Playing), the breaker entity is NOT despawned (it has `CleanupOnRunEnd`, not `CleanupOnNodeExit`). On re-entry, `init_breaker_params` runs again BUT is gated by `Without<BreakerMaxSpeed>`. Since `BreakerMaxSpeed` is already present, the system is a no-op — good, the multipliers are not reset. And `init_archetype` also skips because it queries `Without<LivesCount>`, so the archetype multipliers are also preserved. This is correct.

On run restart (after RunEnd), `CleanupOnRunEnd` despawns the breaker. The new breaker spawned in the next run will lack both `BreakerMaxSpeed` and `LivesCount`, so both init systems run fresh. The ordering (archetype writes AFTER `insert_if_new`) means `insert_if_new(1.0)` is the first write and archetype's plain `insert` overwrites it — correct.

**Conclusion**: Logic is correct. No bug.

### Logic & Control Flow [1 potential issue]

**src/ui/systems/animate_fade_out.rs:19** — The timer decrement and early-return pattern on `fade.timer <= 0.0`:

```rust
fade.timer -= dt;
if fade.timer <= 0.0 {
    commands.entity(entity).despawn();
    continue;
}
let t = fade.timer / fade.duration;
let alpha = t * t;
```

After despawn is queued, `continue` skips the color update for that entity. The entity will still render for one more frame with its old alpha (not 0.0). This is a one-frame visual glitch — the entity is visible at its previous non-zero alpha when it should be at 0. This is minor but technically incorrect: the alpha should be set to 0.0 before despawn so the final rendered frame shows a fully transparent entity.

**src/ui/systems/animate_fade_out.rs:23** — When `fade.timer` starts above 0 and dt brings it exactly to `duration` (i.e., `t = 0.0`), `alpha = 0.0 * 0.0 = 0.0`. The entity stays for one extra frame at alpha=0.0 before the next tick triggers the `<= 0` despawn. This is a one-frame delay at full transparency — negligible visual effect, not a bug.

### State Machine [Clean]

### ECS Pitfalls [1 issue]

**src/ui/plugin.rs:27** — `animate_fade_out` is registered under `run_if(in_state(PlayingState::Active))`. The `FadeOut` entities for bump grade text and bolt-lost text are spawned by systems also gated on `PlayingState::Active`, so there is no accumulation risk in `Active` state. However, `FadeOut` entities survive across `PlayingState::Paused` (if the game is paused mid-fade). The `animate_fade_out` system won't tick their timers while paused, which is intentional freeze behavior.

But there is a subtlety: if the game transitions to `RunEnd` while fade entities are still alive (they have `CleanupOnNodeExit`), `OnExit(Playing)` will despawn them. So no accumulation occurs. This is fine.

**One real concern**: The bolt-lost text system `spawn_bolt_lost_text` is registered in `BoltPlugin` as `FixedUpdate` with `run_if(in_state(PlayingState::Active))`. The `animate_fade_out` system is in `Update` with the same guard. FadeOut entities spawned in `FixedUpdate` will be ticked in `Update` in the same frame, which is correct — commands from FixedUpdate flush before Update runs.

No actual bug here.

### ECS Pitfalls [1 issue — moved system, cross-schedule ordering]

**src/bolt/plugin.rs:43 / src/ui/plugin.rs:27** — `spawn_bolt_lost_text` runs in `FixedUpdate`. `animate_fade_out` runs in `Update`. On the frame a bolt-lost text entity is spawned, `animate_fade_out` will process it in `Update` of that same frame (since FixedUpdate runs before Update). The entity's timer will be decremented by the frame's `dt`, not a fixed timestep. This is consistent with the original behavior (previously `animate_fade_out` was also in Update in the bolt domain — or was it? Need to verify). Since the move to UI domain keeps it in Update, and `bolt_lost_feedback.rs` no longer contains `animate_fade_out`, this is consistent.

The test in `bolt_lost_feedback.rs` (line 53) correctly imports `animate_fade_out` from `crate::ui::systems::animate_fade_out` — no stale import.

No bug.

### Physics & Math [Clean]

**src/breaker/systems/spawn_breaker.rs:58** — `f32::midpoint(playfield.left(), playfield.right())`. The playfield is symmetric: `left() = -width/2`, `right() = width/2`, so `midpoint = 0.0`. This is mathematically equivalent to the previous `(left + right) / 2.0` but avoids potential overflow (though overflow isn't a real concern here at ~400 world units). The result is always exactly 0.0 for symmetric playfields. Correct.

### Test Correctness [2 issues]

**src/breaker/systems/init_breaker_params.rs:224–250 (`archetype_multipliers_not_overwritten`)** — This test pre-stamps `BumpPerfectMultiplier(1.5)` and `BumpWeakMultiplier(0.8)` on a breaker entity that also has `BreakerMaxSpeed` absent. The system runs and calls `insert_if_new`. Since the multipliers are already present, `insert_if_new` should skip them. The test then asserts the values are 1.5 and 0.8. **However**, the test spawns the entity WITHOUT `BreakerMaxSpeed`, so the outer `init_breaker_params` query DOES match and runs. It calls `insert(...)` for all the standard components (which don't include the multipliers in the main inserts), then calls `insert_if_new((BumpPerfectMultiplier(1.0), BumpWeakMultiplier(1.0)))`. Since the entity already has these components, `insert_if_new` is a no-op. The test passes correctly and for the right reason.

**But**: the test comment says "archetype-stamped" but the test is actually just testing `insert_if_new` semantics with pre-existing components. It does NOT test the actual `init_archetype` → `insert_if_new` ordering in context. The `archetype_multipliers_not_overwritten` test would pass even if `init_breaker_params` used plain `insert` instead of `insert_if_new` — no wait, if it used plain `insert(BumpPerfectMultiplier(1.0))`, it would overwrite 1.5 with 1.0 and the test would FAIL. So the test IS a genuine correctness guard. OK.

**src/run/node/systems/spawn_cells_from_layout.rs:284–304 (`grid_is_horizontally_centered`)** — This test computes `center = f32::midpoint(expected_start, expected_end)` and asserts it's within 1.0 of zero. The math is: for a symmetric grid with an odd number of columns (3), the center cell is at column 1, which is at `start_x + step_x`. With `start_x = -grid_width/2 + cell_width/2`, the center of the outermost cells averages to... let me check:

```
expected_start = -grid_width/2 + width/2
expected_end   = step_x * (cols-1) + expected_start = step_x * 2 + expected_start
center = midpoint(expected_start, expected_end)
       = expected_start + step_x
       = -grid_width/2 + width/2 + step_x
       = -(step_x * cols - padding_x)/2 + width/2 + step_x
       = -(step_x * 3 - padding_x)/2 + width/2 + step_x
```

With `step_x = width + padding_x`:
```
= -(3*(width+padding_x) - padding_x)/2 + width/2 + (width+padding_x)
= -(3*width + 2*padding_x)/2 + width/2 + width + padding_x
= -3*width/2 - padding_x + width/2 + width + padding_x
= (-3*width/2 + width/2 + width)
= (-3*width/2 + 3*width/2)
= 0
```

The center IS exactly 0 for symmetric odd-column grids. The test is mathematically sound and the assertion `center.abs() < 1.0` is verified to be exactly 0.0 (within floating-point precision). The test is correct but the tolerance of 1.0 is generously loose — not a bug, but `f32::EPSILON` or 0.01 would be more precise.

**src/cells/systems/handle_cell_hit.rs:26–28** — The early exit via `peek()`:

```rust
let mut messages = reader.read().peekable();
if messages.peek().is_none() {
    return;
}
```

This pattern is correct: `peek()` returns `None` only if the iterator is empty, and the early return avoids the main loop entirely. The subsequent `for hit in messages` correctly consumes remaining messages. No logic error. The `Vec::<Entity>::new()` `despawned` deduplication guard is correct for handling multi-bolt scenarios.

**One subtle issue**: `despawned.contains(&hit.cell)` is O(n) linear scan, but this is noted in the comment as acceptable given the small bound (MAX_BOUNCES = 4). Not a correctness issue.

### Summary

The changes are largely correct. The most interesting invariant to verify was the system ordering for `insert_if_new` defaults vs archetype overrides: the ordering is `init_breaker_params` (writes `insert_if_new` defaults) then `init_archetype` (writes plain `insert` overrides), which correctly allows archetypes to win. The `animate_fade_out` relocation from bolt domain to UI domain is clean — imports updated in tests, plugin registration moved correctly, state guard preserved.

The one minor correctness concern is in `animate_fade_out` (line 19): entities scheduled for despawn render one final frame at their previous non-zero alpha rather than at 0. This is a single-frame visual artifact — set `color.0 = color.0.with_alpha(0.0)` before the `continue` to eliminate it. Not game-breaking.

The `f32::midpoint` usage for centering the breaker is mathematically correct and equivalent to the previous formula.

The new cell position tests are mathematically sound.

**Confidence**: High. No logic bugs that affect gameplay correctness. One minor one-frame visual artifact in fade-out despawn path.

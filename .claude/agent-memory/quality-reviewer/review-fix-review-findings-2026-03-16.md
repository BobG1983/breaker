---
name: review-fix-review-findings-2026-03-16
description: Quality review of fix/review-findings branch changes — init_breaker_params, animate_fade_out, spawn_breaker, handle_cell_hit, spawn_cells_from_layout, state.md, phase-2d plan
type: project
---

# Quality Review — fix/review-findings branch (2026-03-16)

Files reviewed:
- src/breaker/systems/init_breaker_params.rs
- src/ui/systems/animate_fade_out.rs
- src/bolt/systems/bolt_lost_feedback.rs
- src/bolt/plugin.rs
- src/ui/plugin.rs
- src/breaker/systems/spawn_breaker.rs
- src/cells/systems/handle_cell_hit.rs
- src/run/node/systems/spawn_cells_from_layout.rs
- docs/architecture/state.md
- docs/plan/phase-2/phase-2d-screens-and-ui.md

## Idioms — 2 issues

### handle_cell_hit.rs:26-29 — Peek-then-iterate pattern is idiomatic but slightly redundant

```rust
let mut messages = reader.read().peekable();
if messages.peek().is_none() {
    return;
}
let mut despawned = Vec::<Entity>::new();
for hit in messages {
```

The early-exit guard avoids allocating `despawned` when there are no messages — a reasonable micro-optimization.
However, the comment says "Small vec suffices — MAX_BOUNCES = 4" which makes the allocation cost negligible.
The `.peekable()` / `.peek().is_none()` pattern is correct but the motivation is weak: the `for` loop body
would be a no-op anyway with zero messages. Not a bug, but worth noting as borderline over-engineering.
Low priority — do not change unless the early-exit guard was added to paper over a correctness concern.

### spawn_cells_from_layout.rs:288 — `layout.clone()` passed to `test_app` but only `.cols` is read after

```rust
let mut app = test_app(layout.clone());
app.update();
// Then layout.cols is read from the clone
#[allow(clippy::cast_precision_loss)]
let cols_f = layout.cols as f32;
```

`layout` is moved into `test_app` but then `layout.cols` is read from the *original* local binding.
The clone is on the test-side only (not production code), and `NodeLayout` likely derives `Clone` for
exactly this use, so this is a test-quality observation rather than a production idiom issue. The fix
would be to either pass a reference to `test_app` or read `cols` before the move. As written it works
but the clone is required only because of the move — could be avoided with a small refactor. Minor.

## Vocabulary — Clean.

All identifiers reviewed comply with the project vocabulary. No `brick`, `ball`, `paddle`, `level`,
`powerup`, `score`, `hit`, or `strike` terms found in any of the changed files.

`BumpPerfectMultiplier`, `BumpWeakMultiplier` — correct.
`spawn_bolt_lost_text`, `BoltLost`, `FadeOut` — correct.
`handle_cell_hit`, `CellDestroyed`, `BoltHitCell` — correct vocabulary (`hit` appears only in
`BoltHitCell` which is an established message name, not a new identifier).
`reset_breaker`, `center_x`, `f32::midpoint` usage — all fine.

## Test Coverage — 2 gaps

### animate_fade_out.rs — No test for near-zero timer (sub-frame boundary)

The despawn guard is `fade.timer <= 0.0`. The existing `fade_out_despawns_when_expired` test only
exercises the case where timer *starts* at exactly 0.0 before the first tick. There is no test for
the case where the timer starts slightly above 0 (e.g., 0.001) and a normal 16ms tick crosses the
boundary — the entity should despawn on the tick that crosses zero. This is a real edge case: an
entity spawned with a very short `FADE_DURATION` might not despawn correctly if the timer/duration
mismatch is introduced. Low-risk given the simple arithmetic, but worth adding.

### spawn_cells_from_layout.rs — No test for unrecognized alias in registry

The production code at line 48-50 silently skips cells whose alias is not in `CellTypeRegistry`:
```rust
let Some(def) = registry.types.get(&alias) else {
    continue;
};
```
There is no test asserting that an unrecognized alias (e.g., `'X'` not in the registry) produces
zero entities for that slot — only the recognized aliases spawn cells. This is a defensive branch
that could mask layout data errors silently. A test would prevent regressions if `continue` is
accidentally replaced with a panic or if the registry lookup contract changes.

## Documentation — 1 issue

### spawn_breaker.rs:54-57 — `reset_breaker` doc does not mention `f32::midpoint`

The doc comment says "Returns breaker to center" but does not call out that centering is now
computed via `f32::midpoint(playfield.left(), playfield.right())` rather than a hardcoded `0.0`.
This is non-obvious: a reader might wonder why the constant isn't just `0.0` (the playfield is
symmetric around zero). A one-line inline comment explaining why `midpoint` is used rather than
`0.0` would make the intent clear — specifically that it is robust to non-zero-centered playfields
(future-proofing) or that it correctly handles the case if `PlayfieldConfig` ever supports offset.

The function-level doc is otherwise clean and complete.

## Summary

This is a clean, well-structured changeset. The migration of `animate_fade_out` from the bolt domain
to the UI domain is handled correctly — the bolt tests now import the system from `crate::ui::systems`
(cross-domain import in test code only, which is acceptable). The new `insert_if_new` pattern for
default bump multipliers is the right approach and the four new tests covering identity values,
archetype preservation, and skip-if-initialized are all behavior-focused and non-brittle.

The new cell position tests in `spawn_cells_from_layout.rs` are the strongest addition in this
changeset — they verify concrete coordinates against the formula, cover both full and sparse layouts,
and check spacing. The `f32::midpoint` usage in `reset_breaker` is the correct, readable approach.

Priority order for fixes:
1. Add test for unrecognized registry alias (test gap — prevents silent data errors)
2. Add test for sub-frame timer expiry in `animate_fade_out` (low risk, but closes the boundary case)
3. Add inline comment to `reset_breaker` explaining `f32::midpoint` vs `0.0` (docs)
4. The `layout.clone()` in position tests can stay as-is — the clone is cheap and the test is clear

**Why:** No production code is missing these — they are defensive test/doc improvements. None of the
issues above are blockers.

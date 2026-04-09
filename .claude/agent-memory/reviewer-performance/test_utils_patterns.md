---
name: test_utils patterns
description: Patterns in shared/test_utils.rs and how domain helpers relate to it — for calibrating test-infrastructure performance reviews
type: project
---

`TestAppBuilder` wraps a minimal Bevy `App` with a fluent builder API. It is the shared test
infrastructure in `breaker-game/src/shared/test_utils.rs`.

**Why:** Graph shows `test_app()` at 196 edges and `tick()` at 120 edges — but these are
*domain-local* `fn test_app()` helpers in ~42 `tests/helpers.rs` files, NOT the `TestAppBuilder`
type itself. Most domains define their own `test_app()` directly on `App` (raw Bevy calls) rather
than going through `TestAppBuilder`. `TestAppBuilder` is used only in the self-tests inside
`test_utils.rs`.

**How to apply:** When reviewing test_utils.rs, focus on whether `TestAppBuilder` is
well-structured for future adoption. The per-domain duplication of `test_app()` + `tick()` is a
separate pattern concern (not a performance issue). The `tick()` in `test_utils.rs` is duplicated
verbatim in at least the bolt_cell_collision, handle_cell_hit, and shockwave helpers — that's an
adoption gap, not a performance problem.

Key patterns observed:
- `clear_messages<M>` in `First`, `collect_messages<M>` in `Last` — correct schedule placement
- `with_message_capture` idempotency guard uses `contains_resource` — O(1) check, fine
- `in_state_node_playing` does 4 sequential `app.update()` calls — this is the cost per test
  that needs state navigation; acceptable but callers should be aware
- `tick()` reads `Time<Fixed>::timestep()` dynamically — correct, not hardcoded
- `MessageCollector<M>` backed by `Vec<M>` — no pre-allocation, grows with messages per tick;
  fine at test scale (messages per tick is tiny)
- Domain-local `test_app()` helpers do NOT use `TestAppBuilder` — they build raw `App` directly
  with `.add_plugins(MinimalPlugins)` and manual wiring

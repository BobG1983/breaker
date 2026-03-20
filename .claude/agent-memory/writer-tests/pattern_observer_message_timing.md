---
name: Observer Message Timing in Tests
description: When testing observers that write Messages, flush commands before ticking so MessageReader can see the writes
type: feedback
---

# Observer Message Timing in Tests

**Why:** When an observer writes messages via `MessageWriter<T>`, and a capture system reads them via `MessageReader<T>` in `FixedUpdate`, the trigger command must be flushed before `tick()`. Otherwise, the observer fires during `app.update()` inside `tick()`, but the message buffering context means `MessageReader` cannot see those writes in the same update cycle.

**How to apply:** When writing test helpers that trigger observers via `commands.trigger()`, always call `app.world_mut().flush()` between the trigger and `tick()`:

```rust
fn trigger_my_observer(app: &mut App, ...) {
    app.world_mut()
        .commands()
        .trigger(MyEvent { ... });
    app.world_mut().flush();  // Observer fires here, writes messages
    tick(app);                // FixedUpdate capture system reads messages
}
```

If `flush() + tick()` still doesn't work (double-buffering), try `tick() + tick()` instead (observer fires in first tick, reader sees in second).

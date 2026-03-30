---
name: build-failure-bolt-breaker-query (RESOLVED)
description: RESOLVED — bolt_breaker_collision/system.rs E0308 nested query destructure fix applied; code at line 127-142 uses correct 2-level tuple nesting
type: project
---

RESOLVED. The fix described here is present in the current codebase.

`breaker-game/src/bolt/systems/bolt_breaker_collision/system.rs:127-142` uses:
```rust
let Ok((
    breaker_entity,
    (breaker_position, breaker_tilt, ...),
)) = breaker_query.single()
```

No action needed. This file can be deleted on next audit.

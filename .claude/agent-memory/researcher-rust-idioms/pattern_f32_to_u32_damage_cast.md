---
name: f32-to-u32 damage cast pattern (DELETED — stale)
description: DELETED — codebase migrated damage to f32; u32/u16 cast pattern no longer exists
type: project
---

This file has been retired. The codebase migrated `BASE_BOLT_DAMAGE` from `u32 = 10` to
`f32 = 10.0` (see `breaker-game/src/shared/mod.rs`). The `take_boosted_damage` method no
longer exists; damage is computed directly in f32 arithmetic via `take_damage(f32)` in
`cells/components.rs`. The u16/u32 clamp-and-cast pattern described here is no longer
present in the codebase.

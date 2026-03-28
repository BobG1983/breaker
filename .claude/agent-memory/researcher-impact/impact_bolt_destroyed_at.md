---
name: BoltDestroyedAt impact map
description: All references to BoltDestroyedAt across the breaker-game crate — definition, registration, tests, producers, consumers
type: project
---

# BoltDestroyedAt Impact Map (2026-03-28)

## Summary

`BoltDestroyedAt` is a stub message with zero producers and zero consumers in live code.
It was created as a forward-looking placeholder for a two-phase bolt destruction pipeline.
`bridge_bolt_death` (the intended sender) does not exist yet.
Safe to remove: yes. No functional behavior depends on it.

## References

### Definition
- `breaker-game/src/bolt/messages.rs:73` — struct definition, `#[derive(Message, Clone, Debug)]`, body is entirely commented out (no fields)

### Message Registration (producer)
- `breaker-game/src/bolt/plugin.rs:44` — `app.add_message::<BoltDestroyedAt>()` in `BoltPlugin::build`
  - Imported at line 34: `use crate::bolt::messages::{BoltDestroyedAt, BoltSpawned, RequestBoltDestroyed};`

### Tests
- `breaker-game/src/bolt/messages.rs:142` — `bolt_destroyed_at_debug_format` test in `#[cfg(test)] mod tests`
  - Constructs `BoltDestroyedAt {}` and asserts debug output contains "BoltDestroyedAt"

### Producers (systems that send/write)
- None. The doc comment says "Sent by `bridge_bolt_death`" but that system does not exist.
  The trigger bridges in `src/effect/triggers/died.rs` and `src/effect/triggers/death.rs` are stubs
  with placeholder comments ("Wave 8").

### Consumers (systems that read)
- None. The doc comment on the struct says "no consumers — positional data for future VFX"
  (as recorded in `docs/architecture/messages.md:33`).
  `cleanup_destroyed_bolts` reads only `RequestBoltDestroyed`, not `BoltDestroyedAt`.

### Documentation
- `docs/architecture/messages.md:33` — listed in the Active Messages table:
  `BoltDestroyedAt { position }` | effect (bridge_bolt_death) | *(no consumers — positional data for future VFX)*
  Note: the doc shows `{ position }` field but the actual struct has that field commented out.

## Removal Checklist

To remove `BoltDestroyedAt` completely:

1. `breaker-game/src/bolt/messages.rs:73-77` — delete struct definition
2. `breaker-game/src/bolt/messages.rs:142-146` — delete test `bolt_destroyed_at_debug_format`
3. `breaker-game/src/bolt/plugin.rs:34` — remove `BoltDestroyedAt` from the use statement
4. `breaker-game/src/bolt/plugin.rs:44` — remove `.add_message::<BoltDestroyedAt>()` line
5. `docs/architecture/messages.md:33` — remove row from Active Messages table

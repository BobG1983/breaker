---
name: pattern_entity_generational_ids_testing
description: Bevy uses generational entity IDs — stale HashMap entries do not cause false violations on "recycled" IDs; only memory leaks result
type: feedback
---

Bevy's `Entity` type encodes both an index and a generation counter. When an entity is despawned and a new one is spawned, the new entity has the SAME index but a HIGHER generation. Therefore:

- `old_entity != new_entity` even if the slot was reused
- `HashMap<Entity, T>` lookups with the new entity's ID will NOT match stale entries from the old entity
- Stale entries in a `Local<HashMap<Entity, T>>` that are never cleaned up cause **memory leaks**, NOT false positive violations

**Why:** A spec for `check_valid_breaker_state` described "false violations on recycled entity IDs" — but in practice this is impossible with Bevy's generational IDs. The test for "despawn + respawn → no violation" correctly passes even without a `retain` cleanup call, because the new entity's key simply doesn't match any stale entry.

**How to apply:** When a spec mentions "false violations caused by entity ID recycling," skeptically verify whether Bevy's generational IDs actually allow the described collision. The observable bug in such cases is a memory leak (unbounded HashMap growth), not incorrect behavioral output. Design tests to verify the absence of memory leaks (e.g., check HashMap size via a custom system), or note the ambiguity if the spec insists on a behavioral test that passes already.
